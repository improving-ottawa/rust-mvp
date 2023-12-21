mod state;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use uuid::Uuid;

use datum::Datum;
use device::{Device, Id, Name};
use crate::state::State;

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so some historic data is held
/// in memory.
pub struct Controller {
    name: Name,
    id: Id,
    state: Arc<Mutex<State>>
}

impl Device for Controller {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            name: Name::new("controller"),
            id: Id::new("controller"),
            state: Arc::new(Mutex::new(State::new()))
        }
    }
}

impl Controller {
    /// Starts the discovery process as well as polling sensors
    fn run(&self) -> std::io::Result<()> {
        let discover_sensors = Arc::clone(&self.state);
        let discover_actuators = Arc::clone(&self.state);

        // Continuously attempt to discover new devices on the network
        std::thread::spawn(move || loop {
            {
                let mut ctrl = discover_sensors.lock().unwrap();
                ctrl.discover("_sensor").unwrap();
            }

            // FIXME why are we sleeping here? Let's remove this
            std::thread::sleep(Duration::from_secs(30));
        });

        std::thread::spawn(move || loop {
            {
                let mut ctrl = discover_actuators.lock().unwrap();
                ctrl.discover("_actuator").unwrap();
            }

            // FIXME why are we sleeping here? Let's remove this
            std::thread::sleep(Duration::from_secs(30));
        });

        // Cycle through and poll the Sensors, if the return Datum is outside a defined range
        // send a command off to the Actuator
        let self_api_clone = Arc::clone(&self.state);

        // FIXME I think this loop below needs to happen inside the State, like the discover() loop
        std::thread::spawn(move || loop {

            // acquire a mutex lock on the state
            let mut ctrl = self_api_clone.lock().unwrap();

            // loop over all known sensors
            for (id, )



            // Create a temp vec to hold the data history as there is a lock on the controller and
            // we can't populate the history until the lock is released.
            let mut data_history: Vec<(Id, SensorHistory)> = Vec::new();
            {
                let ctrl = self_api_clone.lock().unwrap();

            }

            // Once we have exited the scope where we acquired the data and send commands
            // its safe to acquire lock on ctrl again and update its data history
            let mut ctrl = self_api_clone.lock().unwrap();
            for (id, history) in data_history {
                ctrl.data.insert(id, history);
            }
            std::thread::sleep(Duration::from_secs(5));
        });

        Ok(())
    }
}
