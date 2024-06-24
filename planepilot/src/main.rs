use std::collections::HashMap;

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use futures_timer::Delay;
use planepilot::types::{self, HorizontalModes, VerticalModes};
use tokio::time::Duration;

use serde_json::{Number, Value};

#[tokio::main]
async fn main() {
    println!("Planepilot started");

    match planepilot::run_app().await {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e)
        }
    }
}
