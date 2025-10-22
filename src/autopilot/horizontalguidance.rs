use crate::types::{AutoPilotHorizontalMetrics, Command, HorizontalModes, PlaneStateStruct};

pub(super) async fn execute_horizontal_guidance(
    dt: &f64,
    app_state: &mut std::sync::MutexGuard<'_, crate::types::AppState>,
) -> anyhow::Result<Option<Command>> {
    let plane_state_struct: PlaneStateStruct = app_state.return_plane_state_struct().await;

    //determine the target roll
    let (target_roll, target_roll_rate): (f64, f64) = match app_state
        .autopilot_state
        .horizontal_guidance
        .horizontal_mode
    {
        HorizontalModes::Standby => {
            // early return as we are not doing any aileron here
            return Ok(None);
        }
        HorizontalModes::Heading => {
            let heading_error = app_state
                .autopilot_state
                .horizontal_guidance
                .heading_setpoint
                - plane_state_struct.heading;

            //only add to integral if heading error is less than 15 degrees
            if heading_error.abs() < 15.0 {
                app_state
                    .autopilot_state
                    .horizontal_guidance
                    .heading_error_integral += heading_error * dt;
            }

            // determine target roll
            let target_roll = ((heading_error
                * app_state.autopilot_state.control_constants.heading_error_p)
                + (app_state
                    .autopilot_state
                    .horizontal_guidance
                    .heading_error_integral
                    * app_state.autopilot_state.control_constants.heading_error_i))
                .clamp(
                    -app_state.autopilot_state.control_constants.max_roll,
                    app_state.autopilot_state.control_constants.max_roll,
                );

            let roll_error = target_roll - plane_state_struct.roll;

            // determine the target roll rate
            let target_roll_rate: f64 = (roll_error
                * app_state
                    .autopilot_state
                    .control_constants
                    .heading_roll_error_d)
                .clamp(
                    -app_state.autopilot_state.control_constants.max_roll_rate,
                    app_state.autopilot_state.control_constants.max_roll_rate,
                );

            (target_roll, target_roll_rate)
        }

        HorizontalModes::WingsLevel => {
            // target roll and target roll rate is zero, as we are trying to keep wings level
            (0.0, 0.0)
        }
    };

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
}
