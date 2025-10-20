use crate::types::{AutoPilotHorizontalMetrics, Command, HorizontalModes, PlaneStateStruct};

pub(super) async fn execute_horizontal_guidance(
    dt: &f64,
    app_state: &mut std::sync::MutexGuard<'_, crate::types::AppState>,
    tx: &tokio::sync::mpsc::Sender<Command>,
) -> anyhow::Result<Option<Command>> {
    //determine the target roll
    let target_roll: f64 = match app_state
        .autopilot_state
        .horizontal_guidance
        .horizontal_mode
    {
        HorizontalModes::Standby => {
            // early return as we are not doing any aileron here
            return Ok(None);
        }
        HorizontalModes::Heading => 0.0,

        HorizontalModes::WingsLevel => {
            // target roll is zero, as we are trying to keep wings level
            0.0
        }
    };

    let p: f64 = app_state.autopilot_state.control_constants.roll_p;
    let d: f64 = app_state.autopilot_state.control_constants.roll_d;

    return Ok(None);

    // TODO
    //want to rewrite to
    // first determin the right roll, and then have an inner loop control the ailerons to achieve that roll. can merge the heading and the wings level inner loop

    /*

    let plane_state_struct: PlaneStateStruct => app_state.return_plane_state_struct().await;

    let aileron: f64 = (-(plane_state_struct.roll * p + plane_state_struct.roll_rate * d))
        .clamp(
            -app_state.autopilot_state.control_constants.max_aileron,
            app_state.autopilot_state.control_constants.max_aileron,
        );

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

    app_state.autopilot_state.horizontal_control_metrics = horizontal_metrics;
    return Ok(Some(Command::new_aileron(aileron)));

    */
}
