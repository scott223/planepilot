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

pub enum PacketType {
    Data,
    PREL,
}

#[derive(Debug)]
pub struct Command {
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

#[derive(Debug, Clone, Copy)]
pub enum CommandType {
    Throttle,
    Aileron,
    Elevator,
    ResetPosition,
}
