use crate::AutoPilotHorizontalMetrics;

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
    match auto_pilot_state.horizontal_guidance.horizontal_mode {
        HorizontalModes::Standby => {
            //println!("Horizontal mode standby, no autopilot input for ailerons");
        }
        HorizontalModes::Heading => {
            let kp: f64 = auto_pilot_state.control_constants.heading_error_p;
            let kd: f64 = auto_pilot_state.control_constants.heading_roll_error_d;

            let heading_error: f64 =
                auto_pilot_state.horizontal_guidance.heading_setpoint - plane_state_struct.heading;
            
            
            let target_roll_angle: f64 = (kp * heading_error).clamp(-auto_pilot_state.control_constants.max_roll, auto_pilot_state.control_constants.max_roll);

            let roll_error: f64 = target_roll_angle - plane_state_struct.roll;
            let target_roll_rate: f64 = (kd * roll_error).clamp(-auto_pilot_state.control_constants.max_roll_rate, auto_pilot_state.control_constants.max_roll_rate);
            let roll_rate_error: f64 = target_roll_rate - plane_state_struct.roll_rate;

            let p: f64 = auto_pilot_state.control_constants.roll_p;
            let d: f64 = auto_pilot_state.control_constants.roll_d;
            let i: f64 = auto_pilot_state.control_constants.roll_i;

            // heading error not used now
            app_state_proxy
                .add_to_heading_error_integral(heading_error * dt)
                .await?;

            app_state_proxy
                .add_to_roll_error_integral(roll_error * dt)
                .await?;

            let aileron: f64 = (roll_error * p
                + roll_rate_error * d
                + auto_pilot_state.horizontal_guidance.roll_error_integral * i)
                .clamp(-auto_pilot_state.control_constants.max_aileron, auto_pilot_state.control_constants.max_aileron);

            tracing::event!(tracing::Level::TRACE,
                "Heading mode - heading [deg]: {:.4}, heading error [deg]: {:.4}, target_roll_angle [deg]: {:.4}, roll [deg]: {:.4}, roll_error: {:.4}, target roll rate [deg]: {:.4}, roll rate [deg/s]: {:.4}, roll_rate_error: {:.4}, aileron [0-1]: {:.4}",
                plane_state_struct.heading, heading_error, target_roll_angle, plane_state_struct.roll, roll_error, target_roll_rate, plane_state_struct.roll_rate, roll_rate_error, aileron
            );

            let horizontal_metrics: AutoPilotHorizontalMetrics = AutoPilotHorizontalMetrics {
                heading: plane_state_struct.heading,
                heading_target: auto_pilot_state.horizontal_guidance.heading_setpoint,
                heading_error: heading_error,
                roll_angle: plane_state_struct.roll,
                roll_angle_target: target_roll_angle,
                roll_angle_error: roll_error,
                roll_angle_rate: plane_state_struct.roll_rate,
                roll_angle_rate_target: target_roll_rate,
                roll_angle_rate_error: roll_rate_error,
                aileron_setpoint: aileron,
            };

            app_state_proxy
                .update_horizontal_control_metrics(horizontal_metrics)
                .await?;
            send_command(app_state_proxy, &client, CommandType::Aileron, aileron).await?;
        }
        HorizontalModes::WingsLevel => {
            let p: f64 = auto_pilot_state.control_constants.roll_p;
            let d: f64 = auto_pilot_state.control_constants.roll_d;

            let aileron: f64 = (-(plane_state_struct.roll * p + plane_state_struct.roll_rate * d))
                .clamp(-auto_pilot_state.control_constants.max_aileron, auto_pilot_state.control_constants.max_aileron);

            tracing::event!(tracing::Level::TRACE,
                "Wings level mode - roll [deg]: {:.4}, roll_rate [deg/s]: {:.4}, aileron [0-1]: {:.4}",
                plane_state_struct.roll, plane_state_struct.roll_rate, aileron
            );

            let horizontal_metrics = AutoPilotHorizontalMetrics {
                heading: plane_state_struct.heading,
                heading_target: 0.,
                heading_error: 0.,
                roll_angle: plane_state_struct.roll,
                roll_angle_target: 0.,
                roll_angle_error: plane_state_struct.roll,
                roll_angle_rate: plane_state_struct.roll_rate,
                roll_angle_rate_target: 0.,
                roll_angle_rate_error: plane_state_struct.roll_rate,
                aileron_setpoint: aileron,
            };
            
            app_state_proxy.update_horizontal_control_metrics(horizontal_metrics).await?;
            send_command(app_state_proxy, &client, CommandType::Aileron, aileron).await?;
        }
    }

    Ok(())
}
