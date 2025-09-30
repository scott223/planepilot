use serde_json::{Number, Value};
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use anyhow::anyhow;
use tracing::{event, Level};

//use super::types::{AppStateProxy, Command, CommandType, PacketType};
//use super::xplanedatamap::{data_map, DataIndex, DataType};

const FLOAT_LEN: usize = 4;
const IP_ADRR: &str = "127.0.0.1";
const LISTENING_PORT: &str = "49101";
const SENDING_PORT: &str = "49000";

// Listens to mpsc channel if commands are received, and turn them into an UDP packet to send to xplane
/*
pub(super) async fn listen_to_send_commands(mut rx: mpsc::Receiver<Command>) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(IP_ADRR.to_owned() + ":49100")
        .await
        .map_err(|e| panic!("error: {:?}", e))
        .unwrap();

    loop {
        // wait untill we received a command message
        // the -999.0 means we dont change the value and xplane leaves it alone

        while let Some(c) = rx.recv().await {
            let packet: Vec<u8> = match c.return_command_type() {
                // following command will create DATA packets that set certain values
                CommandType::Elevator => create_packet(
                    PacketType::Data,
                    Some(&[c.return_value(), -999.0_f64, -999.0_f64]),
                    Some(8_u8),
                )?,
                CommandType::Aileron => create_packet(
                    PacketType::Data,
                    Some(&[-999.0_f64, c.return_value(), -999.0_f64]),
                    Some(8_u8),
                )?,
                CommandType::Throttle => {
                    create_packet(PacketType::Data, Some(&[c.return_value()]), Some(25_u8))?
                }
                // this command creates a PREL packet that will position the plane
                CommandType::ResetPosition => create_packet(PacketType::PREL, None, None)?,
            };

            // send the packet over the UdpSocket

            let len = socket
                .send_to(&packet, IP_ADRR.to_owned() + ":" + SENDING_PORT)
                .await
                .map_err(|e| {
                    event!(
                        Level::ERROR,
                        "Error sending command package. Command: {:?}, and error: {:?}",
                        c,
                        e
                    );
                });

            event!(
                Level::TRACE,
                "Command package sent (len: {:?}): {:?}",
                len,
                c
            );

            // we add a 15 ms delay here, to make sure we dont saturate the xplane UDP interface
            let _ = tokio::time::sleep(Duration::from_millis(15)).await;
        }
    }
}

*/

// Listen to xplane UDP packets, and update the state accordingly
pub async fn listen_to_xplane() -> anyhow::Result<()> {
    let socket = UdpSocket::bind(IP_ADRR.to_owned() + ":" + LISTENING_PORT).await?;
    let mut buf: [u8; 1024] = [0_u8; 1024];

    // get the datamap that contains the mapping of data packages into the state
    let data_map: Vec<DataIndex> = data_map();

    loop {
        let (len, _src) = socket.recv_from(&mut buf).await?;

        // check if we get a DATA packet
        if &buf[0..4] == b"DATA" {
            for sentence in buf[5..len].chunks(36) {
                // there is a 0 after DATA, and only take part of the buffer that actually contains the udp packet [5..len]
                // take the data and translate the bytes to floats
                let values = match translate_bytes_to_floats(
                    &sentence[FLOAT_LEN..FLOAT_LEN + 8 * FLOAT_LEN]
                        .try_into()
                        .map_err(|e| {
                            anyhow!(
                                "need 32 bytes ({}), got len {:?}",
                                e,
                                sentence[FLOAT_LEN..FLOAT_LEN + 8 * FLOAT_LEN].len()
                            )
                        })?,
                ) {
                    Ok(v) => v,
                    Err(e) => return Err(anyhow!("Error translating values: {}", e)),
                };

                // use the values and datamap to make a hashmap that contains key-value pairs for the state
                let values = map_values(sentence[0], values, &data_map).map_err(|e| {
                    event!(
                        Level::ERROR,
                        "Error while mapping the floats to the plane state: {:?}",
                        e
                    );
                });

                //send a signal to the app state - via the proxy - to update the state
                //app_state_proxy.add_value_to_state(values.unwrap()).await?;
            }
        }
    }
    Ok(());
}

// Translates 32 bytes to 8 floats
fn translate_bytes_to_floats(data_bytes: &[u8; 8 * 4]) -> anyhow::Result<Vec<f32>> {
    let mut floats: Vec<f32> = Vec::with_capacity(8);

    // chop the data in chunks and convert to a f32, using Little Endian
    for f in data_bytes.chunks(4) {
        floats.push(f32::from_le_bytes(match f.try_into() {
            Ok(b) => b,
            Err(r) => return Err(r.into()),
        }));
    }

    Ok(floats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_bytes_to_floats() {
        let bytes: [u8; 8 * 4] = [
            0x00, 0x00, 0x48, 0x41, 0x00, 0x00, 0x48, 0x41, 0x00, 0x00, 0x48, 0x41, 0x00, 0x00,
            0x48, 0x41, 0x00, 0x00, 0x48, 0x41, 0x00, 0x00, 0x48, 0x41, 0x00, 0x00, 0x48, 0x41,
            0x00, 0x00, 0x48, 0x41,
        ];
        let vec: Vec<f32> = [12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5, 12.5].to_vec();

        assert_eq!(translate_bytes_to_floats(&bytes).unwrap(), vec);
    }
}

// Maps values into the plane_state, based on the data map index
// E.g. [['roll',float], ['pitch',float]] will map the first two floats into the plane state to roll and pitch

fn map_values(
    packet_index: u8,
    values: Vec<f32>,
    data_map: &[DataIndex],
) -> anyhow::Result<BTreeMap<String, Value>> {
    let mut plane_state: BTreeMap<String, Value> = BTreeMap::new();

    match data_map.iter().find(|m| m.index == packet_index) {
        Some(m) => {
            for (index, data) in m.data.iter().enumerate() {
                match data.data_type {
                    DataType::Float => {
                        let mut value: f64 = values[index] as f64;

                        // apply a transformation, if there is one, e.g. going from rad/s to deg/s
                        if let Some(t) = data.transformation {
                            value *= t;
                        }

                        plane_state.insert(
                            data.name.to_string(),
                            Value::Number(Number::from_f64(value).unwrap()),
                        );
                    }
                    DataType::Boolean => {
                        let b: bool = values[index] == 1.0_f32; //convert a 1 to true
                        plane_state.insert(data.name.to_string(), Value::Bool(b));
                    }
                    DataType::Integer => { // todo
                    }
                    DataType::Empty => { // here we don't need to anything
                    }
                };
            }

            // add the current update timestamp to plane_state
            plane_state.insert(
                "last_updated_timestamp".to_string(),
                Value::Number(chrono::Utc::now().timestamp_millis().into()),
            );
        }
        None => {
            event!(
                Level::DEBUG,
                "Packet received but index not found in the datamap. Index: {:?}",
                packet_index
            );
        }
    };

    Ok(plane_state)
}

// create a packet of different types (e.g. DATA, PREL) with the values given

fn create_packet(
    packet_type: PacketType,
    values: Option<&[f64]>,
    index: Option<u8>,
) -> anyhow::Result<Vec<u8>> {
    let mut packet: Vec<u8> = vec![0; 41]; // we use a 41 bytes package as a starting point, as most package will be the data packet

    match packet_type {
        PacketType::Data => {
            if index.is_none() {
                return Err(anyhow!("Need index for data package"));
            }

            if values.is_none() {
                return Err(anyhow!("Need values for data package"));
            }

            packet[0..4].copy_from_slice(b"DATA");
            packet[5] = index.unwrap();

            for (chunk, &value) in packet[9..].chunks_mut(4).zip(values.unwrap()) {
                chunk.copy_from_slice(&(value as f32).to_le_bytes());
            }

            Ok(packet)
        }
        PacketType::PREL => {
            /*
            PREL + \0 upfront (5 bytes)

            struct PREL_struct
            {
                init_flt_enum	type_start; 4 bytes
                xint			p_idx; 4
                xchr			apt_id[idDIM]; 8
                xint			apt_rwy_idx; 4
                xint			apt_rwy_dir; 4
                xdob			dob_lat_deg; 8
                xdob			dob_lon_deg; 8
                xdob			dob_ele_mtr; 8
                xdob			dob_psi_tru; 8
                xdob			dob_spd_msc; 8
            };

            */

            packet = vec![0; 69]; // a PREL packet is 69 bytes

            packet[0..4].copy_from_slice(b"PREL");
            packet[5] = 6_u8; // TYPE START = loc_specify_lle

            // put our plane above Amsterdam at 3000 ft
            let values: [f64; 5] = [52.3676, 4.9041, 914.4, 0.0, 51.444];

            for (chunk, value) in packet[29..].chunks_mut(8).zip(values) {
                // Little Endian again
                chunk.copy_from_slice(&value.to_le_bytes());
            }

            event!(Level::TRACE, "PREL reset packet prepared: {:?}", packet);
            Ok(packet)
        }
    }
}
