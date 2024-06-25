use std::collections::HashMap;

use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Debug)]
pub(super) enum SpecificErrors {
    PlaneConnectorNotReachable,
    PlaneConnectorReturnedError,
    StateNotUpdatedRecently,
}

impl std::fmt::Display for SpecificErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaneConnectorNotReachable => write!(f, "PlaneConnector is not reachable"),
            Self::PlaneConnectorReturnedError => write!(f, "PlaneConnector returned an error"),
            Self::StateNotUpdatedRecently => {
                write!(f, "State is available, but not updated")
            }
        }
    }
}

impl std::error::Error for SpecificErrors {}
pub(super) struct AppState {
    receiver: mpsc::Receiver<StateSignal>,
    auto_pilot_state: AutoPilotState,
    plane_state: HashMap<String, Value>,
}

pub(super) struct PlaneStateStruct {
    pub v_ind: f64,
    pub altitude_msl: f64,
    pub roll: f64,
    pub roll_rate: f64,
    pub pitch: f64,
    pub pitch_rate: f64,
    pub heading: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct AutoPilotState {
    pub are_we_flying: bool,
    pub vertical_guidance: VerticalGuidance,
    pub horizontal_guidance: HorizontalGuidance,
}

impl AutoPilotState {
    pub fn new() -> Self {
        AutoPilotState {
            are_we_flying: false,
            vertical_guidance: VerticalGuidance {
                vertical_mode: VerticalModes::Standby,
                velocity_setpoint: 100.0,
                velocity_standby: 80.0,
                altitude_setpoint: 3000.0,
                altitude_standby: 3500.0,
                energy_error_integral: 0.0,
                pitch_error_integral: 0.0,
            },
            horizontal_guidance: HorizontalGuidance {
                horizontal_mode: HorizontalModes::Standby,
                heading_setpoint: 90.0,
                heading_standby: 120.0,
                heading_error_integral: 0.0,
            },
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VerticalGuidance {
    pub vertical_mode: VerticalModes,
    pub velocity_setpoint: f64,
    pub velocity_standby: f64,
    pub altitude_setpoint: f64,
    pub altitude_standby: f64,
    pub energy_error_integral: f64,
    pub pitch_error_integral: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HorizontalGuidance {
    pub horizontal_mode: HorizontalModes,
    pub heading_setpoint: f64,
    pub heading_standby: f64,
    pub heading_error_integral: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum VerticalModes {
    Standby,
    TECS,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HorizontalModes {
    Standby,
    WingsLevel,
    Heading,
}

impl AppState {
    pub fn new(rx: mpsc::Receiver<StateSignal>) -> Self {
        AppState {
            auto_pilot_state: AutoPilotState::new(),
            plane_state: HashMap::new(),
            receiver: rx,
        }
    }

    // Process incoming commands asynchronously
    pub async fn process(mut self) {
        while let Some(signal) = self.receiver.recv().await {
            match signal {
                StateSignal::SetFlying {
                    are_we_flying,
                    result_sender,
                } => {
                    self.auto_pilot_state.are_we_flying = are_we_flying;
                    let _ = result_sender.send(true);
                }
                StateSignal::ClearPlaneState { result_sender } => {
                    self.plane_state.clear();
                    let _ = result_sender.send(true);
                }
                StateSignal::SetPlaneState {
                    plane_state,
                    result_sender,
                } => {
                    self.plane_state = plane_state;
                    let _ = result_sender.send(true);
                }
                StateSignal::ReturnPlaneState { result_sender } => {
                    let _ = result_sender.send(self.plane_state.clone());
                }
                StateSignal::ReturnPlaneStateStruct { result_sender } => {
                    let state_struct = PlaneStateStruct {
                        v_ind: self.plane_state.get("Vind").unwrap().as_f64().unwrap(),
                        altitude_msl: self
                            .plane_state
                            .get("altitude_msl")
                            .unwrap()
                            .as_f64()
                            .unwrap(),
                        roll: self.plane_state.get("roll").unwrap().as_f64().unwrap(),
                        roll_rate: self.plane_state.get("P").unwrap().as_f64().unwrap(),
                        pitch: self.plane_state.get("pitch").unwrap().as_f64().unwrap(),
                        pitch_rate: self.plane_state.get("Q").unwrap().as_f64().unwrap(),
                        heading: self.plane_state.get("heading").unwrap().as_f64().unwrap(),
                    };

                    let _ = result_sender.send(state_struct);
                }
                StateSignal::ReturnAutoPilotState { result_sender } => {
                    let _ = result_sender.send(self.auto_pilot_state.clone());
                }
                StateSignal::SetStandbyHeading {
                    standby_heading,
                    result_sender,
                } => {
                    self.auto_pilot_state.horizontal_guidance.heading_standby = standby_heading;
                    let _ = result_sender.send(true);
                }
                StateSignal::ActivateStandbyHeading { result_sender } => {
                    self.auto_pilot_state.horizontal_guidance.heading_setpoint =
                        self.auto_pilot_state.horizontal_guidance.heading_standby;
                    let _ = result_sender.send(true);
                }
                StateSignal::SetHorizontalGuidanceToStandbyMode { result_sender } => {
                    self.auto_pilot_state.horizontal_guidance.horizontal_mode =
                        HorizontalModes::Standby;
                    let _ = result_sender.send(true);
                }
                StateSignal::SetHorizontalGuidanceToWingsLevelMode { result_sender } => {
                    self.auto_pilot_state.horizontal_guidance.horizontal_mode =
                        HorizontalModes::WingsLevel;
                    let _ = result_sender.send(true);
                }
                StateSignal::SetHorizontalGuidanceToHeadingMode { result_sender } => {
                    self.auto_pilot_state
                        .horizontal_guidance
                        .heading_error_integral = 0.0;
                    self.auto_pilot_state.horizontal_guidance.horizontal_mode =
                        HorizontalModes::Heading;
                    let _ = result_sender.send(true);
                }
                StateSignal::AddToHeadingErrorIntegral {
                    value,
                    result_sender,
                } => {
                    self.auto_pilot_state
                        .horizontal_guidance
                        .heading_error_integral += value;
                    let _ = result_sender.send(true);
                }
                StateSignal::SetStandbyVelocity {
                    standby_velocity,
                    result_sender,
                } => {
                    self.auto_pilot_state.vertical_guidance.velocity_standby = standby_velocity;
                    let _ = result_sender.send(true);
                }
                StateSignal::ActivateStandbyVelocity { result_sender } => {
                    self.auto_pilot_state.vertical_guidance.velocity_setpoint =
                        self.auto_pilot_state.vertical_guidance.velocity_standby;
                    let _ = result_sender.send(true);
                }
                StateSignal::SetStandbyAltitude {
                    standby_altitude,
                    result_sender,
                } => {
                    self.auto_pilot_state.vertical_guidance.altitude_standby = standby_altitude;
                    let _ = result_sender.send(true);
                }
                StateSignal::ActivateStandbyAltitude { result_sender } => {
                    self.auto_pilot_state.vertical_guidance.altitude_setpoint =
                        self.auto_pilot_state.vertical_guidance.altitude_standby;
                    let _ = result_sender.send(true);
                }
                StateSignal::SetVerticalGuidanceToStandbyMode { result_sender } => {
                    self.auto_pilot_state.vertical_guidance.vertical_mode = VerticalModes::Standby;
                    let _ = result_sender.send(true);
                }
                StateSignal::SetVerticalGuidanceToTECSMode { result_sender } => {
                    self.auto_pilot_state
                        .vertical_guidance
                        .energy_error_integral = 0.0;
                    self.auto_pilot_state.vertical_guidance.pitch_error_integral = 0.0;
                    self.auto_pilot_state.vertical_guidance.vertical_mode = VerticalModes::TECS;
                    let _ = result_sender.send(true);
                }
                StateSignal::AddToEnergyErrorIntegral {
                    value,
                    result_sender,
                } => {
                    self.auto_pilot_state
                        .vertical_guidance
                        .energy_error_integral += value;
                    let _ = result_sender.send(true);
                }
                StateSignal::AddToPitchErrorIntegral {
                    value,
                    result_sender,
                } => {
                    self.auto_pilot_state.vertical_guidance.pitch_error_integral += value;
                    let _ = result_sender.send(true);
                }
            }
        }
    }
}

pub(super) enum StateSignal {
    SetFlying {
        are_we_flying: bool,
        result_sender: oneshot::Sender<bool>,
    },
    SetPlaneState {
        plane_state: HashMap<String, Value>,
        result_sender: oneshot::Sender<bool>,
    },
    ReturnPlaneState {
        result_sender: oneshot::Sender<HashMap<String, Value>>,
    },
    ReturnPlaneStateStruct {
        result_sender: oneshot::Sender<PlaneStateStruct>,
    },
    ClearPlaneState {
        result_sender: oneshot::Sender<bool>,
    },
    ReturnAutoPilotState {
        result_sender: oneshot::Sender<AutoPilotState>,
    },
    SetStandbyHeading {
        standby_heading: f64,
        result_sender: oneshot::Sender<bool>,
    },
    ActivateStandbyHeading {
        result_sender: oneshot::Sender<bool>,
    },
    SetHorizontalGuidanceToStandbyMode {
        result_sender: oneshot::Sender<bool>,
    },
    SetHorizontalGuidanceToHeadingMode {
        result_sender: oneshot::Sender<bool>,
    },
    AddToHeadingErrorIntegral {
        value: f64,
        result_sender: oneshot::Sender<bool>,
    },
    SetHorizontalGuidanceToWingsLevelMode {
        result_sender: oneshot::Sender<bool>,
    },
    SetStandbyVelocity {
        standby_velocity: f64,
        result_sender: oneshot::Sender<bool>,
    },
    ActivateStandbyVelocity {
        result_sender: oneshot::Sender<bool>,
    },
    SetStandbyAltitude {
        standby_altitude: f64,
        result_sender: oneshot::Sender<bool>,
    },
    ActivateStandbyAltitude {
        result_sender: oneshot::Sender<bool>,
    },
    SetVerticalGuidanceToStandbyMode {
        result_sender: oneshot::Sender<bool>,
    },
    SetVerticalGuidanceToTECSMode {
        result_sender: oneshot::Sender<bool>,
    },
    AddToEnergyErrorIntegral {
        value: f64,
        result_sender: oneshot::Sender<bool>,
    },
    AddToPitchErrorIntegral {
        value: f64,
        result_sender: oneshot::Sender<bool>,
    },
}

#[derive(Clone)]
pub(super) struct AppStateProxy {
    pub state_sender: mpsc::Sender<StateSignal>,
}

impl AppStateProxy {
    pub fn new(tx: mpsc::Sender<StateSignal>) -> Self {
        AppStateProxy { state_sender: tx }
    }

    pub async fn set_flying(&self, are_we_flying: bool) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::SetFlying {
                are_we_flying,
                result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    // plane state

    pub async fn get_plane_state(&self) -> anyhow::Result<HashMap<String, Value>> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::ReturnPlaneState { result_sender })
            .await?;

        Ok(result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from state")))
    }

    pub async fn get_plane_state_as_struct(&self) -> anyhow::Result<PlaneStateStruct> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::ReturnPlaneStateStruct { result_sender })
            .await?;

        Ok(result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from state")))
    }

    pub async fn set_plane_state(&self, plane_state: HashMap<String, Value>) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::SetPlaneState {
                plane_state,
                result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn clear_plane_state(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::ClearPlaneState { result_sender })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    // autopilot state

    pub async fn get_auto_pilot_state(&self) -> anyhow::Result<AutoPilotState> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::ReturnAutoPilotState { result_sender })
            .await?;

        Ok(result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from state")))
    }

    // horizontal modes

    pub async fn set_heading_standby(&self, heading: f64) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetStandbyHeading {
                standby_heading: heading,
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn activate_heading_setpoint(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::ActivateStandbyHeading {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn activate_horizontal_standby_mode(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetHorizontalGuidanceToStandbyMode {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn activate_horizontal_wingslevel_mode(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetHorizontalGuidanceToWingsLevelMode {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn activate_horizontal_heading_mode(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetHorizontalGuidanceToHeadingMode {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn add_to_heading_error_integral(&self, value: f64) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::AddToHeadingErrorIntegral {
                value,
                result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    // vertical modes

    pub async fn set_velocity_standby(&self, velocity: f64) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetStandbyVelocity {
                standby_velocity: velocity,
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn activate_velocity_setpoint(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::ActivateStandbyVelocity {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn set_altitude_standby(&self, altitude: f64) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetStandbyAltitude {
                standby_altitude: altitude,
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn activate_alititude_setpoint(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::ActivateStandbyAltitude {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn activate_vertical_standby_mode(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetVerticalGuidanceToStandbyMode {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    #[allow(non_snake_case)]
    pub async fn activate_vertical_TECS_mode(&self) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::SetVerticalGuidanceToTECSMode {
                result_sender: result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn add_to_energy_error_integral(&self, value: f64) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::AddToEnergyErrorIntegral {
                value,
                result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }

    pub async fn add_to_pitch_error_integral(&self, value: f64) -> anyhow::Result<()> {
        let (result_sender, result_receiver) = oneshot::channel();

        self.state_sender
            .send(StateSignal::AddToPitchErrorIntegral {
                value,
                result_sender,
            })
            .await?;

        match result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from auto pilot state"))
        {
            true => return Ok(()),
            _ => return Err(anyhow!("Error with receiving result from autopilot state")),
        }
    }
}

pub struct Command {
    pub command_type: CommandType,
    pub value: f64,
}

pub enum CommandType {
    Aileron,
    Elevator,
    Throttle,
}
