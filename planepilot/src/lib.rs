use std::collections::HashMap;

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use futures_timer::Delay;
use serde_json::{Number, Value};
use tokio::{sync::mpsc, time::Duration};
use types::{
    AppState, AppStateProxy, HorizontalModes, PlaneStateStruct, SpecificErrors, VerticalModes,
};

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

        let auto_pilot_state: types::AutoPilotState =
            app_state_proxy.get_auto_pilot_state().await?;
        // let plane_state: HashMap<String, Value> = app_state_proxy.get_plane_state().await?;

        if auto_pilot_state.are_we_flying {
            let client = reqwest::Client::new();
            let plane_state_struct: PlaneStateStruct =
                app_state_proxy.get_plane_state_as_struct().await?;

            match auto_pilot_state.vertical_guidance.vertical_mode {
                VerticalModes::Standby => {}
                VerticalModes::TECS => {
                    let dt: f64 = 0.1;

                    // calculate specific (so no mass term) energy target
                    let target_kinetic: f64 = 0.5
                        * (auto_pilot_state.vertical_guidance.velocity_setpoint * 0.5111)
                        * (auto_pilot_state.vertical_guidance.velocity_setpoint * 0.5111); //speed to m/s
                    let target_potential: f64 =
                        (auto_pilot_state.vertical_guidance.altitude_setpoint * 0.304) * 9.81; // altitude to m

                    let target_energy: f64 = target_kinetic + target_potential;

                    let kinetic: f64 = 0.5
                        * (plane_state_struct.v_ind * 0.51111)
                        * (plane_state_struct.v_ind * 0.51111);
                    let potential: f64 = (plane_state_struct.altitude_msl * 0.304) * 9.81;
                    let energy: f64 = kinetic + potential;

                    let energy_error: f64 = target_energy - energy;

                    app_state_proxy.add_to_energy_error_integral(energy_error * dt).await?;

                    let ke: f64 = 0.0010;
                    let ks = 0.0000001;
                    let thr_cruise = 0.48 + target_energy * ks;

                    let ki = 0.0001;

                    let throttle = ke * energy_error
                        + thr_cruise
                        + auto_pilot_state.vertical_guidance.energy_error_integral * ki;

                    println!(
                        "TEC mode - alitude [ft]: {:.4}, Vind [kt]: {:.4}, energy_error: {:.4}, integral: {:.4}, throttle: {:.4}",
                        plane_state_struct.altitude_msl, plane_state_struct.v_ind, energy_error, auto_pilot_state.vertical_guidance.energy_error_integral, throttle
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("throttle".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(throttle).unwrap()),
                    );

                    let _res = match client
                        .post("http://localhost:3100/api/v1/command")
                        .json(&map)
                        .send()
                        .await
                    {
                        Ok(_res) => {}
                        Err(_) => {}
                    };

                    let kpitch: f64 = -1.5;

                    let target_pitch: f64 =
                        ((auto_pilot_state.vertical_guidance.velocity_setpoint
                            - plane_state_struct.v_ind)
                            * kpitch)
                            .clamp(-15.0, 15.0);
                    let pitch_error = target_pitch - plane_state_struct.pitch;

                    app_state_proxy.add_to_pitch_error_integral(pitch_error * dt).await?;

                    let kpr = 0.3;

                    let target_pitch_rate = (pitch_error * kpr).clamp(-3.0, 3.0);
                    let pitch_rate_error = target_pitch_rate - plane_state_struct.pitch_rate;

                    let kelevator = 0.15;
                    let kdelevator = 0.0;
                    let kielevator: f64 = 0.015;

                    let elevator = (kelevator * pitch_error
                        + kdelevator * pitch_rate_error
                        + kielevator * auto_pilot_state.vertical_guidance.pitch_error_integral)
                        .clamp(-0.3, 0.3);

                    println!(
                        "TEC mode - pitch [deg]: {:.4}, target_pitch [deg]: {:.4}, pitch_error [deg]: {:.4}, pitch_rate: {:.4}, target_pitch_rate: {:.4}, pitch_rate_error: {:.4}, elevator {:.4}",
                        plane_state_struct.pitch, target_pitch, pitch_error, plane_state_struct.pitch_rate, target_pitch_rate, pitch_rate_error, elevator
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("elevator".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(elevator).unwrap()),
                    );

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

            match auto_pilot_state.horizontal_guidance.horizontal_mode {
                HorizontalModes::Standby => {
                    println!("Horizontal mode standby, no autopilot input for ailerons");
                }
                HorizontalModes::Heading => {
                    let target_heading: f64 = 170.0;

                    let kp: f64 = 0.4;
                    let kd = 0.2;

                    let heading_error: f64 = target_heading - plane_state_struct.heading;
                    let target_roll_angle: f64 = (kp * heading_error).clamp(-30.0, 30.0);
                    let roll_error: f64 = target_roll_angle - plane_state_struct.roll;
                    let target_roll_rate: f64 = (kd * roll_error).clamp(-3.0, 3.0);
                    let roll_rate_error: f64 = target_roll_rate - plane_state_struct.roll_rate;

                    let p: f64 = 0.01;
                    let d: f64 = 0.01;

                    // TODO: add heading integral

                    let aileron: f64 =
                        (roll_error * p + roll_rate_error * d).clamp(-MAX_AILERON, MAX_AILERON);

                    println!(
                        "Heading mode - heading [deg]: {:.4}, heading error [deg]: {:.4}, target_roll_angle [deg]: {:.4}, roll [deg]: {:.4}, roll_error: {:.4}, target roll rate [deg]: {:.4}, roll rate [deg/s]: {:.4}, roll_rate_error: {:.4}, aileron [0-1]: {:.4}",
                        plane_state_struct.heading, heading_error, target_roll_angle, plane_state_struct.roll, roll_error, target_roll_rate, plane_state_struct.roll_rate, roll_rate_error, aileron
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("aileron".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(aileron).unwrap()),
                    );

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
                    let p: f64 = 0.01;
                    let d: f64 = 0.01;

                    let aileron: f64 = (-(plane_state_struct.roll * p
                        + plane_state_struct.roll_rate * d))
                        .clamp(-MAX_AILERON, MAX_AILERON);

                    println!(
                        "Wings level mode - roll [deg]: {:.4}, roll_rate [deg/s]: {:.4}, aileron [0-1]: {:.4}",
                        plane_state_struct.roll, plane_state_struct.roll_rate, aileron
                    );

                    let mut map: HashMap<String, Value> = HashMap::new();
                    map.insert("command".to_string(), Value::String("aileron".to_string()));
                    map.insert(
                        "value".to_string(),
                        Value::Number(Number::from_f64(aileron).unwrap()),
                    );

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
