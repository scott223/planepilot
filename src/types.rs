use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

// first-order low-pass filter
pub struct LowPassFilter {
    b0: f64,
    b1: f64,
    a1: f64,
    x1: f64, // previous input
    y1: f64, // previous output
}

impl LowPassFilter {
    pub fn new(sample_rate: f64, tau: f64) -> Self {
        let t = 1.0 / sample_rate;
        let b0 = t / (t + 2.0 * tau);
        let b1 = b0;
        let a1 = (t - 2.0 * tau) / (t + 2.0 * tau);

        Self {
            b0,
            b1,
            a1,
            x1: 0.0,
            y1: 0.0,
        }
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 - self.a1 * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }

    pub fn get_latest(self) -> f64 {
        self.y1.clone()
    }
}

// App state - has a receiver to receive signals and a trait to respond to it, no memory sharing
#[derive(Debug, Clone)]
pub(super) struct AppState {
    pub plane_state: BTreeMap<String, Value>,
    pub autopilot_state: AutoPilotState,
}

// struct to use in autopilot
pub(super) struct PlaneStateStruct {
    pub v_ind: f64,
    pub altitude_msl: f64,
    pub vpath: f64,
    pub roll: f64,
    pub roll_rate: f64,
    pub pitch: f64,
    pub pitch_rate: f64,
    pub gload_axial: f64,
    pub heading: f64,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            plane_state: BTreeMap::new(),
            autopilot_state: AutoPilotState::new(),
        }
    }

    pub async fn add_vales(&mut self, values: BTreeMap<String, Value>) {
        for (key, val) in values.iter() {
            self.plane_state.insert(key.to_string(), val.clone());
        }

        // add a timestamp
        self.plane_state.insert(
            "last_updated_timestamp".to_string(),
            Value::Number(
                serde_json::Number::from_f64(chrono::Utc::now().timestamp_millis() as f64).unwrap(),
            ),
        );
    }

    pub async fn return_plane_state_struct(&self) -> PlaneStateStruct {
        //dbg!(self.plane_state.clone());
        let state_struct = PlaneStateStruct {
            v_ind: self.plane_state.get("Vind").unwrap().as_f64().unwrap(),
            altitude_msl: self
                .plane_state
                .get("altitude_msl")
                .unwrap()
                .as_f64()
                .unwrap(),
            vpath: self.plane_state.get("vpath").unwrap().as_f64().unwrap(),
            roll: self.plane_state.get("roll").unwrap().as_f64().unwrap(),
            roll_rate: self.plane_state.get("P").unwrap().as_f64().unwrap(),
            pitch: self.plane_state.get("pitch").unwrap().as_f64().unwrap(),
            pitch_rate: self.plane_state.get("Q").unwrap().as_f64().unwrap(),
            gload_axial: self
                .plane_state
                .get("Gload_axial")
                .unwrap()
                .as_f64()
                .unwrap(),
            heading: self
                .plane_state
                .get("heading_true")
                .unwrap()
                .as_f64()
                .unwrap(),
        };

        state_struct
    }

    /*
    // Process incoming commands asynchronously
    pub async fn process(mut self) {
        while let Some(signal) = self.receiver.recv().await {
            match signal {
                StateSignal::ReturnPlaneState { result_sender } => {
                    let mut state: BTreeMap<String, Value> = BTreeMap::new();

                    for (key, val) in self.plane_state.iter() {
                            state.insert(
                                key.to_string(),
                                val[0].clone(),
                            );
                        }

                    let _ = result_sender.send(state.clone());
                }
                StateSignal::ReturnFilteredPlaneState { result_sender } => {
                    let mut state: BTreeMap<String, Value> = BTreeMap::new();

                    for (key, val) in self.plane_state_filtered.iter() {
                            state.insert(
                                key.to_string(),
                                Value::Number(serde_json::Number::from_f64(val.get_latest()).unwrap())
                            );
                        }

                    let _ = result_sender.send(state.clone());
                }

                StateSignal::UpdatePlaneState {
                    state,
                    result_sender,
                } => {
                    for (key, val) in state.iter() {

                        self.plane_state
                            .entry(key.to_string())
                            .and_modify(|f| {
                                f.insert(0, val.clone());

                                // make sure it never grows larger than a set size
                                if f.len() > 100 {
                                    f.pop();
                                }
                            })
                            .or_insert(vec![val.clone()]);


                            self.plane_state_filtered
                            .entry(key.to_string())
                            .and_modify(|f| {

                                if val.is_f64() {

                                f.process(val.as_f64().unwrap());

                                                            }
                            })
                            .or_insert(LowPassFilter::new(30.0, 0.1));

                        // add the current update timestamp to plane_state
                        // TODO
                        self.plane_state.insert(
                            "last_updated_timestamp".to_string(),
                            vec![Value::Number(chrono::Utc::now().timestamp_millis().into())],
                        );



                    }
                    let _ = result_sender.send(true);
                }
            }
        }
    }

    */
}

// Commands

// Define the types of commands that can be sent to xplane
#[derive(Debug, Clone, Copy)]
pub(crate) enum CommandType {
    Throttle,
    Aileron,
    Elevator,
    ResetPosition,
}

// Define a command to be sent to xplane
#[derive(Debug)]
pub(crate) struct Command {
    command_type: CommandType,
    value: f64,
}

impl Command {
    pub fn new_throttle(v: f64) -> Self {
        Command {
            command_type: CommandType::Throttle,
            value: v.clamp(0.0, 1.0),
        }
    }

    pub fn new_aileron(v: f64) -> Self {
        Command {
            command_type: CommandType::Aileron,
            value: v.clamp(-1.0, 1.0),
        }
    }

    pub fn new_elevator(v: f64) -> Self {
        Command {
            command_type: CommandType::Elevator,
            value: v.clamp(-1.0, 1.0),
        }
    }

    pub fn new_reset() -> Self {
        Command {
            command_type: CommandType::ResetPosition,
            value: 0.0_f64,
        }
    }

    pub fn return_command_type(&self) -> CommandType {
        self.command_type.clone()
    }

    pub fn return_value(&self) -> f64 {
        self.value
    }
}
#[derive(Debug, Default, Serialize, Clone)]
pub(super) struct AutoPilotState {
    pub are_we_flying: bool,
    #[serde(flatten)]
    pub vertical_guidance: VerticalGuidance,
    #[serde(flatten)]
    pub horizontal_guidance: HorizontalGuidance,
    #[serde(flatten)]
    pub control_constants: AutoPilotConstants,
    #[serde(flatten)]
    pub horizontal_control_metrics: AutoPilotHorizontalMetrics,
    #[serde(flatten)]
    pub vertical_control_metrics: AutoPilotVerticalMetrics,
}

#[derive(Debug, Default, Serialize, Clone)]
pub(super) struct AutoPilotVerticalMetrics {
    pub altitude_msl: f64,
    pub altitude_target: f64,
    pub altitude_error: f64,
    pub velocity: f64,
    pub velocity_target: f64,
    pub velocity_error: f64,
    pub kinetic_energy: f64,
    pub kinetic_energy_target: f64,
    pub potential_energy: f64,
    pub potential_energy_target: f64,
    pub energy: f64,
    pub energy_target: f64,
    pub energy_error: f64,
    pub pitch: f64,
    pub pitch_target: f64,
    pub pitch_error: f64,
    pub pitch_rate: f64,
    pub pitch_rate_target: f64,
    pub pitch_rate_error: f64,
    pub elevator_setpoint: f64,
}

#[derive(Debug, Default, Serialize, Clone)]
pub(super) struct AutoPilotHorizontalMetrics {
    pub heading: f64,
    pub heading_target: f64,
    pub heading_error: f64,
    pub roll_angle: f64,
    pub roll_angle_target: f64,
    pub roll_angle_error: f64,
    pub roll_angle_rate: f64,
    pub roll_angle_rate_target: f64,
    pub roll_angle_rate_error: f64,
    pub aileron_setpoint: f64,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub(super) struct AutoPilotConstants {
    pub heading_error_p: f64,
    pub heading_error_i: f64,
    pub heading_roll_error_d: f64,
    pub roll_p: f64,
    pub roll_d: f64,
    pub roll_i: f64,
    pub tecs_cruise_throttle_slope: f64,
    pub tecs_cruise_throttle_base: f64,
    pub tecs_energy_p: f64,
    pub tecs_energy_i: f64,
    pub pitch_error_p: f64,
    pub pitch_rate_error_p: f64,
    pub elevator_p: f64,
    pub elevator_d: f64,
    pub elevator_i: f64,
    pub max_aileron: f64,
    pub max_roll: f64,
    pub max_roll_rate: f64,
    pub max_elevator: f64,
    pub max_pitch: f64,
    pub max_pitch_rate: f64,
}

impl AutoPilotConstants {
    pub fn new() -> Self {
        AutoPilotConstants {
            heading_error_p: 0.4,
            heading_error_i: 0.1,
            heading_roll_error_d: 0.2,
            roll_p: 0.01,
            roll_d: 0.01,
            roll_i: 0.001,
            tecs_cruise_throttle_slope: 0.0000001,
            tecs_cruise_throttle_base: 0.48,
            tecs_energy_p: 0.001,
            tecs_energy_i: 0.001,
            pitch_error_p: -1.5,
            pitch_rate_error_p: 0.3,
            elevator_p: 0.15,
            elevator_d: 0.015,
            elevator_i: 0.0015,
            max_aileron: 0.3,
            max_roll: 30.0,
            max_roll_rate: 3.0,
            max_elevator: 0.5,
            max_pitch: 15.0,
            max_pitch_rate: 15.0,
        }
    }

    pub fn from_file() -> Self {
        let path = Path::new("./constants.json");
        let mut file = File::open(path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let json: AutoPilotConstants = serde_json::from_str(&data).unwrap();
        json
    }

    pub fn _to_file(&self) -> anyhow::Result<()> {
        let path = Path::new("./constants.json");
        let mut file = std::fs::File::create(path)?;
        let list_as_json = serde_json::to_string(self).unwrap();

        file.write_all(list_as_json.as_bytes())
            .expect("Cannot write to the file!");

        Ok(())
    }
}

impl AutoPilotState {
    pub fn new() -> Self {
        AutoPilotState {
            are_we_flying: false,
            vertical_guidance: VerticalGuidance {
                vertical_mode: VerticalModes::TECS,
                velocity_setpoint: 100.0,
                velocity_standby: 80.0,
                altitude_setpoint: 3100.0,
                altitude_standby: 3500.0,
                energy_error_integral: 0.0,
                pitch_error_integral: 0.0,
            },
            horizontal_guidance: HorizontalGuidance {
                horizontal_mode: HorizontalModes::Heading,
                heading_setpoint: 90.0,
                heading_standby: 120.0,
                heading_error_integral: 0.0,
                roll_error_integral: 0.0,
            },
            horizontal_control_metrics: AutoPilotHorizontalMetrics::default(),
            vertical_control_metrics: AutoPilotVerticalMetrics::default(),
            control_constants: AutoPilotConstants::new(),
        }
    }

    pub fn set_autopilot_to_standby_and_clean_parameters(&mut self) {
        self.vertical_guidance.vertical_mode = VerticalModes::Standby;
        self.horizontal_guidance.horizontal_mode = HorizontalModes::Standby;

        self.vertical_guidance.energy_error_integral = 0.0;
        self.vertical_guidance.pitch_error_integral = 0.0;
        self.horizontal_guidance.heading_error_integral = 0.0;
        self.horizontal_guidance.roll_error_integral = 0.0;

        self.horizontal_control_metrics = AutoPilotHorizontalMetrics::default();
        self.vertical_control_metrics = AutoPilotVerticalMetrics::default();
    }
}
#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct VerticalGuidance {
    pub vertical_mode: VerticalModes,
    pub velocity_setpoint: f64,
    pub velocity_standby: f64,
    pub altitude_setpoint: f64,
    pub altitude_standby: f64,
    pub energy_error_integral: f64,
    pub pitch_error_integral: f64,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct HorizontalGuidance {
    pub horizontal_mode: HorizontalModes,
    pub heading_setpoint: f64,
    pub heading_standby: f64,
    pub heading_error_integral: f64,
    pub roll_error_integral: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum VerticalModes {
    Standby,
    TECS,
}

impl Default for VerticalModes {
    fn default() -> Self {
        VerticalModes::Standby
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HorizontalModes {
    Standby,
    WingsLevel,
    Heading,
}

impl Default for HorizontalModes {
    fn default() -> Self {
        HorizontalModes::Standby
    }
}
