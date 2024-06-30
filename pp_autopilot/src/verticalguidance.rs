use super::{
    send_command,
    types::{CommandType, VerticalModes},
};

pub(super) async fn execute_vertical_guidance(
    dt: f64,
    client: &reqwest::Client,
    app_state_proxy: &super::types::AppStateProxy,
    auto_pilot_state: &super::types::AutoPilotState,
    plane_state_struct: &super::types::PlaneStateStruct,
) -> anyhow::Result<()> {
    const MAX_ELEVATOR: f64 = 0.3;

    const MAX_PITCH: f64 = 15.0;
    const MAX_PITCH_RATE: f64 = 15.0;

    const KNOTS_TO_METERS_PER_SECOND: f64 = 0.514444;
    const FEET_TO_METERS: f64 = 0.3048;
    const GRAVITATIONAL_CONSTANT: f64 = 0.981;

    match auto_pilot_state.vertical_guidance.vertical_mode {
        VerticalModes::Standby => {}
        VerticalModes::TECS => {
            // calculate specific (so no mass term) energy target
            let target_kinetic: f64 = 0.5
                * (auto_pilot_state.vertical_guidance.velocity_setpoint
                    * KNOTS_TO_METERS_PER_SECOND)
                * (auto_pilot_state.vertical_guidance.velocity_setpoint
                    * KNOTS_TO_METERS_PER_SECOND); //speed to m/s
            let target_potential: f64 = (auto_pilot_state.vertical_guidance.altitude_setpoint
                * FEET_TO_METERS)
                * GRAVITATIONAL_CONSTANT; // altitude to m

            let target_energy: f64 = target_kinetic + target_potential;

            let kinetic: f64 = 0.5
                * (plane_state_struct.v_ind * KNOTS_TO_METERS_PER_SECOND)
                * (plane_state_struct.v_ind * KNOTS_TO_METERS_PER_SECOND);
            let potential: f64 =
                (plane_state_struct.altitude_msl * FEET_TO_METERS) * GRAVITATIONAL_CONSTANT;
            let energy: f64 = kinetic + potential;

            let energy_error: f64 = target_energy - energy;

            app_state_proxy
                .add_to_energy_error_integral(energy_error * dt)
                .await?;

            let ke: f64 = auto_pilot_state.control_constants.tecs_energy_p;
            let ks = auto_pilot_state.control_constants.tecs_cruise_throttle_slope;
            let thr_cruise = auto_pilot_state.control_constants.tecs_cruise_throttle_base + target_energy * ks;

            let ki = auto_pilot_state.control_constants.tecs_energy_i;

            let throttle = (ke * energy_error
                + thr_cruise
                + auto_pilot_state.vertical_guidance.energy_error_integral * ki)
                .clamp(0.0, 1.0);

            println!(
                "TEC mode - alitude [ft]: {:.4}, Vind [kt]: {:.4}, energy_error: {:.4}, integral: {:.4}, throttle: {:.4}",
                plane_state_struct.altitude_msl, plane_state_struct.v_ind, energy_error, auto_pilot_state.vertical_guidance.energy_error_integral, throttle
            );

            send_command(&client, CommandType::Throttle, throttle).await?;

            // pitch

            let kpitch: f64 = auto_pilot_state.control_constants.pitch_error_p;

            let target_pitch: f64 = ((auto_pilot_state.vertical_guidance.velocity_setpoint
                - plane_state_struct.v_ind)
                * kpitch)
                .clamp(-MAX_PITCH, MAX_PITCH);
            let pitch_error = target_pitch - plane_state_struct.pitch;

            app_state_proxy
                .add_to_pitch_error_integral(pitch_error * dt)
                .await?;

            let kpr = auto_pilot_state.control_constants.pitch_rate_error_p;

            let target_pitch_rate = (pitch_error * kpr).clamp(-MAX_PITCH_RATE, MAX_PITCH_RATE);
            let pitch_rate_error = target_pitch_rate - plane_state_struct.pitch_rate;

            let kelevator = auto_pilot_state.control_constants.elevator_p;
            let kdelevator = auto_pilot_state.control_constants.elevator_d;
            let kielevator: f64 = auto_pilot_state.control_constants.elevator_i;

            let elevator = (kelevator * pitch_error
                + kdelevator * pitch_rate_error
                + kielevator * auto_pilot_state.vertical_guidance.pitch_error_integral)
                .clamp(-MAX_ELEVATOR, MAX_ELEVATOR);

            println!(
                "TEC mode - pitch [deg]: {:.4}, target_pitch [deg]: {:.4}, pitch_error [deg]: {:.4}, pitch_rate: {:.4}, target_pitch_rate: {:.4}, pitch_rate_error: {:.4}, elevator {:.4}",
                plane_state_struct.pitch, target_pitch, pitch_error, plane_state_struct.pitch_rate, target_pitch_rate, pitch_rate_error, elevator
            );

            send_command(&client, CommandType::Elevator, elevator).await?;
        }
    }

    Ok(())
}
