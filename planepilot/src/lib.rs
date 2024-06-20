use std::collections::HashMap;

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use futures_timer::Delay;
use tokio::{sync::mpsc, time::Duration};
use serde_json::{Number, Value};
use types::{AppState, AppStateProxy, SpecificErrors, VerticalModes};


pub mod types;


pub async fn run_app() -> anyhow::Result<()> {

    tokio::select! {
        _ = run_autopilot() => {

        }
        _ = run_terminal() => {
            println!("terminal completed");
        }
    }

    Ok(())
}

async fn run_autopilot() -> anyhow::Result<()> {

    let (tx_state, rx_state) = mpsc::channel(32);

    // set up the app state and a proxy, that is linked through a channel. we can then clone and share the proxy with all the different procsesses
    let app_state: AppState = AppState::new(rx_state);
    let app_state_proxy: AppStateProxy = AppStateProxy::new(tx_state);    

    loop {
        match update_state().await {
            Ok(plane_state) => {
                app_state_proxy.set_flying(true).await?;
                app_state_proxy.set_plane_state(plane_state).await?;
            }
            Err(e) => {
                app_state_proxy.set_flying(false).await?;
                app_state_proxy.clear_plane_state().await?;

                println!("Error while updating state: {}", e);
            }
        };

        let auto_pilot_state: types::AutoPilotState = app_state_proxy.get_auto_pilot_state().await?;
        let plane_state: HashMap<String, Value> = app_state_proxy.get_plane_state().await?;

        if auto_pilot_state.are_we_flying {
            
            match auto_pilot_state.vertical_guidance.vertical_mode {
                VerticalModes::Standby => {}
                VerticalModes::TECS => {
                    let target_altitude: f64 = 3000.0; // 3000 ft
                    let target_speed = 100.0; // 100 kts
                    let dt: f64 = 0.1;

                    // calculate specific (so no mass term) energy target
                    let target_kinetic: f64 =
                        0.5 * (target_speed * 0.5111) * (target_speed * 0.5111); //speed to m/s
                    let target_potential: f64 = (target_altitude * 0.304) * 9.81; // altitude to m

                    let target_energy: f64 = target_kinetic + target_potential;

                    let vind: f64 = plane_state.get("Vind").unwrap().as_f64().unwrap();
                    let altitude: f64 = plane_state
                        .get("altitude_msl")
                        .unwrap()
                        .as_f64()
                        .unwrap();

                    let kinetic: f64 = 0.5 * (vind * 0.51111) * (vind * 0.51111);
                    let potential: f64 = (altitude * 0.304) * 9.81;
                    let energy: f64 = kinetic + potential;

                    let energy_error: f64 = target_energy - energy;
                    
                    // todo update integral in the autopilot state
                    auto_pilot_state.vertical_guidance.energy_error_integral = auto_pilot_state.vertical_guidance.energy_error_integral + energy_error * dt;

                    let ke: f64 = 0.0010;
                    let ks = 0.0000001;
                    let thr_cruise = 0.48 + target_energy * ks;

                    let ki = 0.0001;

                    let throttle = ke * energy_error + thr_cruise + auto_pilot_state.vertical_guidance.energy_error_integral * ki;

                    println!(
                        "TEC mode - alitude [ft]: {:.4}, Vind [kt]: {:.4}, energy_error: {:.4}, integral: {:.4}, throttle: {:.4}",
                        altitude, vind, energy_error, auto_pilot_state.vertical_guidance.energy_error_integral, throttle
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("throttle".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(throttle).unwrap()),
                    );

                    let client = reqwest::Client::new();

                    let _res = match client
                        .post("http://localhost:3100/api/v1/command")
                        .json(&map)
                        .send()
                        .await
                    {
                        Ok(_res) => {}
                        Err(_) => {}
                    };

                    let pitch: f64 = plane_state
                        .get("pitch")
                        .unwrap()
                        .as_f64()
                        .unwrap();

                    let kpitch: f64 = -1.5;

                    let target_pitch: f64 = ((target_speed - vind) * kpitch).clamp(-15.0, 15.0);
                    let pitch_error = target_pitch - pitch;

                    auto_pilot_state.vertical_guidance.pitch_error_integral += app_state.pitch_error_int + pitch_error * dt;

                    let pitch_rate = app_state.plane_state.get("Q").unwrap().as_f64().unwrap();

                    let kpr = 0.3;

                    let target_pitch_rate = (pitch_error * kpr).clamp(-3.0, 3.0);
                    let pitch_rate_error = target_pitch_rate - pitch_rate;

                    let kelevator = 0.15;
                    let kdelevator = 0.0;
                    let kielevator: f64 = 0.015;

                    let elevator = (kelevator * pitch_error
                        + kdelevator * pitch_rate_error
                        + kielevator * app_state.pitch_error_int)
                        .clamp(-0.3, 0.3);

                    println!(
                        "TEC mode - pitch [deg]: {:.4}, target_pitch [deg]: {:.4}, pitch_error [deg]: {:.4}, pitch_rate: {:.4}, target_pitch_rate: {:.4}, pitch_rate_error: {:.4}, elevator {:.4}",
                        pitch,target_pitch, pitch_error,pitch_rate, target_pitch_rate, pitch_rate_error, elevator
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("elevator".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(elevator).unwrap()),
                    );

                    let client = reqwest::Client::new();

                    let _res = match client
                        .post("http://localhost:3100/api/v1/command")
                        .json(&map)
                        .send()
                        .await
                    {
                        Ok(_res) => {}
                        Err(_) => {}
                    };
                }
            }

            const MAX_AILERON: f64 = 0.3;
            //horizontal mode

            match app_state.horizontal_mode {
                HorizontalModes::Standby => {
                    println!("Horizontal mode standby, no autopilot input for ailerons");
                }
                HorizontalModes::Heading => {
                    let target_heading: f64 = 170.0;

                    let heading = app_state
                        .plane_state
                        .get("heading_true")
                        .unwrap()
                        .as_f64()
                        .unwrap();
                    let roll: f64 = app_state.plane_state.get("roll").unwrap().as_f64().unwrap();
                    let roll_rate = app_state.plane_state.get("P").unwrap().as_f64().unwrap();

                    let kp: f64 = 0.4;
                    let kd = 0.2;

                    let heading_error: f64 = target_heading - heading;
                    let target_roll_angle: f64 = (kp * heading_error).clamp(-30.0, 30.0);
                    let roll_error: f64 = target_roll_angle - roll;
                    let target_roll_rate: f64 = (kd * roll_error).clamp(-3.0, 3.0);
                    let roll_rate_error: f64 = target_roll_rate - roll_rate;

                    let p: f64 = 0.01;
                    let d: f64 = 0.01;

                    let aileron: f64 =
                        (roll_error * p + roll_rate_error * d).clamp(-MAX_AILERON, MAX_AILERON);

                    println!(
                        "Heading mode - heading [deg]: {:.4}, heading error [deg]: {:.4}, target_roll_angle [deg]: {:.4}, roll [deg]: {:.4}, roll_error: {:.4}, target roll rate [deg]: {:.4}, roll rate [deg/s]: {:.4}, roll_rate_error: {:.4}, aileron [0-1]: {:.4}",
                        heading, heading_error, target_roll_angle, roll, roll_error, target_roll_rate, roll_rate, roll_rate_error, aileron
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("aileron".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(aileron).unwrap()),
                    );

                    let client = reqwest::Client::new();

                    let _res = match client
                        .post("http://localhost:3100/api/v1/command")
                        .json(&map)
                        .send()
                        .await
                    {
                        Ok(_res) => {}
                        Err(_) => {}
                    };
                }
                HorizontalModes::WingsLevel => {
                    let roll: f64 = app_state.plane_state.get("roll").unwrap().as_f64().unwrap();
                    let roll_rate = app_state.plane_state.get("P").unwrap().as_f64().unwrap();

                    let p: f64 = 0.01;
                    let d: f64 = 0.01;

                    let aileron: f64 =
                        (-(roll * p + roll_rate * d)).clamp(-MAX_AILERON, MAX_AILERON);

                    println!(
                        "Wings level mode - roll [deg]: {:.4}, roll_rate [deg/s]: {:.4}, aileron [0-1]: {:.4}",
                        roll, roll_rate, aileron
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("aileron".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(aileron).unwrap()),
                    );

                    let client = reqwest::Client::new();

                    let _res = match client
                        .post("http://localhost:3100/api/v1/command")
                        .json(&map)
                        .send()
                        .await
                    {
                        Ok(_res) => {}
                        Err(_) => {}
                    };
                }
            }
        }

        let _ = tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // Ok((s))
}

async fn run_terminal() -> anyhow::Result<()> {
    let mut reader = EventStream::new();

    loop {
        let delay = Delay::new(Duration::from_millis(1_000));

        tokio::select! {
            _ = delay => {
                //println!(".\r");
            },
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        println!("Event::{:?}\r", event);

                        if event == Event::Key(KeyCode::Char('q').into()) {
                            break;
                        }
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }

    Ok(())
}

async fn update_state() -> anyhow::Result<HashMap<String, Value>> {
    let res = match reqwest::get("http://localhost:3100/api/v1/state").await {
        Ok(res) => res,
        Err(_) => {
            return Err(anyhow::Error::new(
                SpecificErrors::PlaneConnectorNotReachable,
            ))
        }
    };

    match res.status() {
        reqwest::StatusCode::OK => {
            let state = res.json::<HashMap<String, Value>>().await?;
            if !state.contains_key("last_updated_timestamp") {
                return Err(anyhow::Error::new(SpecificErrors::StateNotUpdatedRecently));
            }
            return Ok(state);
        }
        _ => {
            return Err(anyhow::Error::new(
                SpecificErrors::PlaneConnectorReturnedError,
            ))
        }
    }
}
