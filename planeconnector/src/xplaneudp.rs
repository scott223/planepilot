use serde_json::{Number, Value};
use std::{collections::HashMap, sync::Arc};

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use anyhow::anyhow;
use tracing::{event, Level};

use crate::types::{Command, CommandType, PlaneState};
use crate::xplanedatamap::{data_map, DataIndex, DataType};

const FLOAT_LEN: usize = 4;

pub async fn listen_to_send_commands(mut rx: mpsc::Receiver<Command>) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:49101").await?;

    loop {
        //wait untill we received a command message

        while let Some(c) = rx.recv().await {
            //todo: built in a 20ms delay - here or in the HTTP?

            let packet: [u8; 41] = match c.return_command_type() {
                CommandType::Elevator => {
                    create_data_command_package(8_u8, &[c.return_value(), -999.0_f64, -999.0_f64])?
                }
                CommandType::Aileron => {
                    create_data_command_package(8_u8, &[-999.0_f64, c.return_value(), -999.0_f64])?
                }
                CommandType::Throttle => create_data_command_package(25_u8, &[c.return_value(); 4])?,
            };

            let len = socket
                .send_to(&packet, "127.0.0.1:49101")
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
                Level::INFO,
                "Command package sent (len: {:?}): {:?}",
                len,
                c
            );
        }
    }
}

pub async fn listen_to_xplane(
    plane_state: &mut Arc<std::sync::RwLock<PlaneState>>,
) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:49100").await?;
    let mut buf: [u8; 1024] = [0_u8; 1024];
    let data_map: Vec<DataIndex> = data_map();

    loop {
        let (len, _src) = socket.recv_from(&mut buf).await?;

        if &buf[0..4] == b"DATA" {
            for sentence in buf[5..len].chunks(36) {

                // there is a 0 after DATA, and only take part of the buffer that actually contains the udp packet [5..len]
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

                {
                    // extra scope to make sure we drop the lock
                    let _ = map_values(
                        sentence[0],
                        values,
                        &data_map,
                        &mut plane_state.write().unwrap().map,
                    )
                    .map_err(|e| {
                        event!(
                            Level::ERROR,
                            "Error while mapping the floats to the plane state: {:?}",
                            e
                        );
                    });
                }
            }
        }
    }
}

/// Translates 32 bytes to 8 floats

fn translate_bytes_to_floats(data_bytes: &[u8; 8 * FLOAT_LEN]) -> anyhow::Result<Vec<f32>> {
    let mut floats: Vec<f32> = Vec::with_capacity(8);

    for f in data_bytes.chunks(FLOAT_LEN) {
        floats.push(f32::from_le_bytes(match f.try_into() {
            Ok(b) => b,
            Err(r) => return Err(r.into()),
        }));
    }

    Ok(floats)
}

/// Maps values into the plane_state, based on the data map index
/// E.g. [['roll',float], ['pitch',float]] will map the first two floats into the plane state to roll and pitch

fn map_values(
    packet_index: u8,
    values: Vec<f32>,
    data_map: &[DataIndex],
    plane_state: &mut HashMap<String, Value>,
) -> anyhow::Result<()> {
    match data_map.iter().find(|m| m.index == packet_index) {
        Some(m) => {
            for (index, data) in m.data.iter().enumerate() {
                match data.data_type {
                    DataType::Float => {
                        plane_state.insert(
                            data.name.to_string(),
                            Value::Number(Number::from_f64(values[index] as f64).unwrap()),
                        );
                    }
                    DataType::Boolean => {
                        let b: bool = values[index] == 1.0_f32; //convert a 1 to true
                        plane_state.insert(data.name.to_string(), Value::Bool(b));
                    }
                    DataType::Integer => { // todo
                    }
                    DataType::Empty => { // here we dont need to anything
                    }
                };
            }
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

    Ok(())
}

fn create_data_command_package(index: u8, values: &[f64]) -> anyhow::Result<[u8; 41]> {
    // create a udp packet [u8; 41] of bytes
    // structure needs to be
    // "DATA" + 0 + index(byte) + 0 0 0 (3x zero bytes) + 8 x floats (as bytes)
    // if not enough floats, the rest will be zeros so to always have 41 bytes
    // note that a -999.0 float value will be interpreted by xplane to ignore the value

    if values.len() > 8 || values.is_empty() {
        return Err(anyhow!("Error creating UDP data command package: cannot package more than 8 or less than 1 floats"));
    }

    let mut packet: [u8; 41] = [0_u8; 41];

    packet[0..4].copy_from_slice(b"DATA");
    packet[5] = index;

    for (chunk, &value) in packet[9..].chunks_mut(4).zip(values) {
        chunk.copy_from_slice(&(value as f32).to_le_bytes());
    }

    Ok(packet)
}
