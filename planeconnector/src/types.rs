use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppState {
    pub plane_state: HashMap<String, Value>,
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