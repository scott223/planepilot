use crate::types::{AutoPilotVerticalMetrics, Command, PlaneStateStruct, VerticalModes};

pub(super) async fn execute_vertical_guidance(
    dt: &f64,
    app_state: &mut std::sync::MutexGuard<'_, crate::types::AppState>,
) -> anyhow::Result<Option<(Command, Command)>> {
    let plane_state_struct: PlaneStateStruct = app_state.return_plane_state_struct().await;

    //determine the target roll
    let target_pitch: f64 = match app_state.autopilot_state.vertical_guidance.vertical_mode {
        VerticalModes::Standby => {
            // early return as we are not doing any aileron here
            return Ok(None);
        }
        VerticalModes::TECS => 1.0,
    };

    return Ok(Some((
        Command::new_throttle(0.55),
        Command::new_elevator(0.0),
    )));

    /*

    let p: f64 = app_state.autopilot_state.control_constants.roll_p;
    let d: f64 = app_state.autopilot_state.control_constants.roll_d;

    let aileron: f64 = ((target_roll - plane_state_struct.roll) * p
        + ((target_roll_rate - plane_state_struct.roll_rate) * d))
        .clamp(
            -app_state.autopilot_state.control_constants.max_aileron,
            app_state.autopilot_state.control_constants.max_aileron,
        );

    tracing::event!(
        tracing::Level::TRACE,
        "{:?} mode | roll [deg]: {:.4}, roll_error [deg]: {:.4}, roll_rate [deg/s]: {:.4}, roll_rate_error [deg/s]: {:.4}, aileron [0-1]: {:.4}",
        app_state.autopilot_state.horizontal_guidance.horizontal_mode,
        plane_state_struct.roll,
        target_roll - plane_state_struct.roll,
        plane_state_struct.roll_rate,
        target_roll_rate - plane_state_struct.roll_rate,
        aileron
    );

    let horizontal_metrics = AutoPilotHorizontalMetrics {
        heading: plane_state_struct.heading,
        heading_target: app_state
            .autopilot_state
            .horizontal_guidance
            .heading_setpoint,
        heading_error: (app_state
            .autopilot_state
            .horizontal_guidance
            .heading_setpoint
            - plane_state_struct.heading),
        roll_angle: plane_state_struct.roll,
        roll_angle_target: target_roll,
        roll_angle_error: target_roll - plane_state_struct.roll,
        roll_angle_rate: plane_state_struct.roll_rate,
        roll_angle_rate_target: target_roll_rate,
        roll_angle_rate_error: target_roll_rate - plane_state_struct.roll_rate,
        aileron_setpoint: aileron,
    };

    app_state.autopilot_state.horizontal_control_metrics = horizontal_metrics;
    return Ok(Some(Command::new_aileron(aileron)));

    */

    return Ok(None);
}
