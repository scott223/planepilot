use tokio::net::UdpSocket;

use anyhow::Result;

const FLOAT_LEN: usize = 4;

pub async fn listen_to_xplane(socket: UdpSocket) -> Result<()> {
    let mut buf: [u8; 1024] = [0_u8; 1024];

    loop {
        let (len, _src) = socket
            .recv_from(&mut buf)
            .await
            .expect("Error whilst receiving UDP packet");

        if &buf[0..4] == b"DATA" {
            for sentence in buf[5..len].chunks(36) {
                // there is a 0 after DATA, and only take part of the buffer that actually contains the udp packet [5..len]
                let values = match translate_to_floats(
                    &sentence[FLOAT_LEN..FLOAT_LEN + 8 * FLOAT_LEN]
                        .try_into()
                        .expect("need 32 bytes"), //start at byte index 4 (first four are used for xplane index)
                ) {
                    Ok(v) => v,
                    Err(e) => panic!("Error translating values: {}", e),
                };

                match sentence[0] {
                    17_u8 => {
                        println!(
                            "pitch: {}, roll: {}, heading: {}",
                            values[0], values[1], values[2]
                        )
                    }
                    20_u8 => {
                        let on_runway: bool = if values[4] == 1.0_f32 { true } else { false }; //convert a 1 to true

                        println!(
                            "latitude: {}, longitude: {}, altitude_msl: {}, altitude_agl: {}, on_runway: {} \n",
                            values[0], values[1], values[2], values[3], on_runway
                        )
                    }
                    _ => {
                        // do nothing
                    }
                };
            }
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
