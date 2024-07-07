use super::{
    send_command,
    types::{CommandType, HorizontalModes},
};

pub(super) async fn execute_horizontal_guidance(
    dt: f64,
    client: &reqwest::Client,
    app_state_proxy: &super::types::AppStateProxy,
    auto_pilot_state: &super::types::AutoPilotState,
    plane_state_struct: &super::types::PlaneStateStruct,
) -> anyhow::Result<()> {
    const MAX_AILERON: f64 = 0.3;
    const MAX_ROLL: f64 = 30.0;
    const MAX_ROLL_RATE: f64 = 3.0;

    match auto_pilot_state.horizontal_guidance.horizontal_mode {
        HorizontalModes::Standby => {
            //println!("Horizontal mode standby, no autopilot input for ailerons");
        }
        HorizontalModes::Heading => {
            let kp: f64 = 0.4;
            let kd: f64 = 0.2;

            let heading_error: f64 =
                auto_pilot_state.horizontal_guidance.heading_setpoint - plane_state_struct.heading;
            let target_roll_angle: f64 = (kp * heading_error).clamp(-MAX_ROLL, MAX_ROLL);

            let roll_error: f64 = target_roll_angle - plane_state_struct.roll;
            let target_roll_rate: f64 = (kd * roll_error).clamp(-MAX_ROLL_RATE, MAX_ROLL_RATE);
            let roll_rate_error: f64 = target_roll_rate - plane_state_struct.roll_rate;

            let p: f64 = 0.01;
            let d: f64 = 0.01;
            let i: f64 = 0.001;

            app_state_proxy
                .add_to_heading_error_integral(heading_error * dt)
                .await?;

            let aileron: f64 = (roll_error * p
                + roll_rate_error * d
                + auto_pilot_state.horizontal_guidance.heading_error_integral * i)
                .clamp(-MAX_AILERON, MAX_AILERON);

            println!(
                "Heading mode - heading [deg]: {:.4}, heading error [deg]: {:.4}, target_roll_angle [deg]: {:.4}, roll [deg]: {:.4}, roll_error: {:.4}, target roll rate [deg]: {:.4}, roll rate [deg/s]: {:.4}, roll_rate_error: {:.4}, aileron [0-1]: {:.4}",
                plane_state_struct.heading, heading_error, target_roll_angle, plane_state_struct.roll, roll_error, target_roll_rate, plane_state_struct.roll_rate, roll_rate_error, aileron
            );

            send_command(&client, CommandType::Aileron, aileron).await?;
        }
        HorizontalModes::WingsLevel => {
            let p: f64 = 0.01;
            let d: f64 = 0.01;

            let aileron: f64 = (-(plane_state_struct.roll * p + plane_state_struct.roll_rate * d))
                .clamp(-MAX_AILERON, MAX_AILERON);

            println!(
                "Wings level mode - roll [deg]: {:.4}, roll_rate [deg/s]: {:.4}, aileron [0-1]: {:.4}",
                plane_state_struct.roll, plane_state_struct.roll_rate, aileron
            );

            send_command(&client, CommandType::Aileron, aileron).await?;
        }
    }

    Ok(())
}
