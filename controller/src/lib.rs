use std::collections::HashMap;

use datum::Datum;
use device::{Device, Id, Name};

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so historic data
#[allow(dead_code)] // remove ASAP
pub struct Controller {
    name: Name,
    id: Id,
    /// Holds data queried from `Sensor`s
    data: HashMap<Id, SensorHistory>,
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
            data: HashMap::new(),
        }
    }
}

impl Controller {
    pub fn new() -> Controller {
        Controller::default()
    }

    /// Loops continually, attempting to discover new devices on the network.
    pub fn discover(group: &str) {
        let mdns = mdns_sd::ServiceDaemon::new().unwrap();
        let service_type = format!("{}._tcp.local.", group);
        let receiver = mdns.browse(service_type.as_str()).unwrap();

        while let Ok(event) = receiver.recv() {
            if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                println!("The controller has discovered: {}", info.get_fullname());
            }
        }
    }
}

/// The `SensorHistory` holds the unique `id`, the user-friendly `name`, and some history of
/// `data` points.
#[allow(dead_code)] // remove ASAP
struct SensorHistory {
    id: Id,
    name: Name,
    data: Vec<Datum>,
}
