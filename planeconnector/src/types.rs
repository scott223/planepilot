use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct AppState {
    pub plane_state: Arc<RwLock<PlaneState>>,
    pub tx_command: Sender<Command>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaneState {
    pub map: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct Command {
    pub command_type: CommandType,
    pub value: f64,
}

#[derive(Debug)]
pub enum CommandType {
    Throttle,
    Aileron,
    Elevator,
}
