use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

// Define the types of commands that can be sent to the AppState actor
#[derive(Debug)]
pub(super) enum StateSignal {
    ReturnPlaneState {
        result_sender: oneshot::Sender<HashMap<String, serde_json::value::Value>>,
    },
    UpdatePlaneState {
        state: HashMap<String, serde_json::value::Value>,
        result_sender: oneshot::Sender<bool>,
    },
}

// App state - has a receiver to receive signals and a trait to respond to it, no memory sharing
pub(super) struct AppState {
    plane_state: HashMap<String, Value>,
    receiver: mpsc::Receiver<StateSignal>,
}

impl AppState {
    pub fn new(receiver: mpsc::Receiver<StateSignal>) -> Self {
        AppState {
            plane_state: HashMap::new(),
            receiver,
        }
    }

    // Process incoming commands asynchronously
    pub async fn process(mut self) {
        while let Some(signal) = self.receiver.recv().await {
            match signal {
                StateSignal::ReturnPlaneState { result_sender } => {
                    let _ = result_sender.send(self.plane_state.clone());
                }
                StateSignal::UpdatePlaneState {
                    state,
                    result_sender,
                } => {
                    for (key, val) in state.iter() {
                        self.plane_state.insert(key.clone(), val.clone());
                    }
                    let _ = result_sender.send(true);
                }
            }
        }
    }
}

// Define the proxy struct for interacting with the actor
#[derive(Clone)]
#[allow(dead_code)]
pub(super) struct AppStateProxy {
    pub service_adresses: (String, String, String),
    pub state_sender: mpsc::Sender<StateSignal>,
    pub command_sender: mpsc::Sender<Command>,
}

impl AppStateProxy {
    pub fn new(service_adresses: &(String, String, String), state_sender: mpsc::Sender<StateSignal>, command_sender: mpsc::Sender<Command>) -> Self {
        AppStateProxy {
            service_adresses: service_adresses.clone(),
            state_sender,
            command_sender,
        }
    }

    // send a command to xplane
    pub async fn send_command(&self, command: Command) -> anyhow::Result<()> {
        match self.command_sender.send(command).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    // send and return state signal and await the result
    pub async fn get_state(&self) -> anyhow::Result<HashMap<String, serde_json::value::Value>> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::ReturnPlaneState { result_sender })
            .await?;
        Ok(result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from state")))
    }

    // Send a value to be added to the state
    pub async fn add_value_to_state(
        &self,
        state: HashMap<String, serde_json::value::Value>,
    ) -> anyhow::Result<bool> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::UpdatePlaneState {
                state,
                result_sender,
            })
            .await?;
        let result = result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive message from state"));

        Ok(result)
    }
}

// define possible UDP packet types, to be send to xplane
pub(super) enum PacketType {
    Data,
    PREL,
}

// Define a command to be sent to xplane
#[derive(Debug)]
pub(super) struct Command {
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
        self.command_type
    }

    pub fn return_value(&self) -> f64 {
        self.value
    }
}

// Define the types of commands that can be sent to xplane

#[derive(Debug, Clone, Copy)]
pub(super) enum CommandType {
    Throttle,
    Aileron,
    Elevator,
    ResetPosition,
}
