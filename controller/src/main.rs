use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use controller::{discover_sensors, query_sensors};
use tokio::signal;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let sensors = Arc::new(Mutex::new(HashMap::new()));

    let discovery_task = tokio::spawn(discover_sensors(sensors.clone()));
    let query_task = tokio::spawn(query_sensors(sensors.clone()));

    tokio::select! {
        _ = discovery_task => {}
        _ = query_task => {}
        _ = signal::ctrl_c() => {
            println!("Controller shutting down....")
        }
    }

    Ok(())
}
