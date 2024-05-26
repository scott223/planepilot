use serde_json::{Number, Value};
use std::{collections::HashMap, sync::Arc};

use tokio::{net::UdpSocket, sync::Mutex};

use anyhow::{anyhow, Result};
use tracing::{event, Level};

use crate::xplanedatamap::{data_map, DataIndex, DataType};

const FLOAT_LEN: usize = 4;

pub async fn listen_to_xplane(app_state: &mut Arc<Mutex<crate::AppState>>) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:49100").await?;
    let mut buf: [u8; 1024] = [0_u8; 1024];
    let data_map: Vec<DataIndex> = data_map();

    loop {
        let (len, _src) = socket.recv_from(&mut buf).await?;

        if &buf[0..4] == b"DATA" {
            for sentence in buf[5..len].chunks(36) {
                // there is a 0 after DATA, and only take part of the buffer that actually contains the udp packet [5..len]
                let values = match translate_to_floats(
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

                let _ = map_values(sentence[0], values, &data_map, &mut app_state.lock().await.plane_state)
                    .map_err(|e| {
                        event!(
                            Level::ERROR,
                            "Error while mapping the floats to the plane state: {:?}",
                            e
                        );
                    });
            }

            //event!(
            //    Level::TRACE,
            //    "New packets received and translated. Plane state: {:?}",
            //    app_state.
            //);
        }
    }
}

fn translate_to_floats(data_bytes: &[u8; 8 * FLOAT_LEN]) -> Result<Vec<f32>> {
    let mut floats: Vec<f32> = Vec::with_capacity(8);

    for f in data_bytes.chunks(FLOAT_LEN) {
        floats.push(f32::from_le_bytes(match f.try_into() {
            Ok(b) => b,
            Err(r) => return Err(r.into()),
        }));
    }

    Ok(floats)
}

fn map_values(
    packet_index: u8,
    values: Vec<f32>,
    data_map: &[DataIndex],
    plane_state: &mut HashMap<String, Value>,
) -> Result<()> {
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
                    DataType::Empty => {}
                };
            }
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
