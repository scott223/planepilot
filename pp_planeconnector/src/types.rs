use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use tokio::sync::{mpsc, oneshot};

// Define the types of commands that can be sent to the AppState actor
#[derive(Debug)]
pub(super) enum StateSignal {
    ReturnPlaneState {
        result_sender: oneshot::Sender<BTreeMap<String, serde_json::value::Value>>,
    },
    ReturnFilteredPlaneState {
        result_sender: oneshot::Sender<BTreeMap<String, serde_json::value::Value>>,
    },
    UpdatePlaneState {
        state: BTreeMap<String, serde_json::value::Value>,
        result_sender: oneshot::Sender<bool>,
    },
}

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

        Self { b0, b1, a1, x1: 0.0, y1: 0.0 }
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
pub(super) struct AppState {
    plane_state: BTreeMap<String, Vec<Value>>,
    plane_state_filtered:  BTreeMap<String, LowPassFilter>, 
    receiver: mpsc::Receiver<StateSignal>,
}

impl AppState {
    pub fn new(receiver: mpsc::Receiver<StateSignal>) -> Self {
        AppState {
            plane_state: BTreeMap::new(),
            plane_state_filtered: BTreeMap::new(),
            receiver,
        }
    }

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
    pub fn new(
        service_adresses: &(String, String, String),
        state_sender: mpsc::Sender<StateSignal>,
        command_sender: mpsc::Sender<Command>,
    ) -> Self {
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
    pub async fn get_state(&self) -> anyhow::Result<BTreeMap<String, serde_json::value::Value>> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::ReturnPlaneState { result_sender })
            .await?;
        Ok(result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive result from state")))
    }

    // send and return state signal and await the result
    pub async fn get_average_state(
        &self,
    ) -> anyhow::Result<BTreeMap<String, serde_json::value::Value>> {
        let (result_sender, result_receiver) = oneshot::channel();
        self.state_sender
            .send(StateSignal::ReturnAveragePlaneState { result_sender })
            .await?;
        Ok(result_receiver
            .await
            .unwrap_or_else(|_| panic!("Failed to receive average state result from state")))
    }

    // Send a value to be added to the state
    pub async fn add_value_to_state(
        &self,
        state: BTreeMap<String, serde_json::value::Value>,
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
