use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

use actuator_temperature::TemperatureActuatorCommand;
use datum::Datum;
use device::{Device, Id, Name};

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so historic data
pub struct Controller {
    name: Name,
    id: Id,
    /// Holds data queried from `Sensor`s
    #[allow(dead_code)] // remove this ASAP
    data: HashMap<Id, SensorHistory>,
    sensor_addresses: HashMap<Id, Address>,
    actuator_addresses: HashMap<Id, Address>,
}

#[derive(Debug)]
struct Address {
    host: String,
    port: String,
}

impl Address {
    fn new(host: &str, port: u16) -> Address {
        Address {
            host: host.to_string(),
            port: port.to_string(),
        }
    }
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
            sensor_addresses: HashMap::new(),
            actuator_addresses: HashMap::new(),
        }
    }
}

pub trait ControllerExtension {
    fn run(&self) -> std::io::Result<()>;
}

impl ControllerExtension for Arc<Mutex<Controller>> {
    /// Starts the discovery process as well as polling sensors
    fn run(&self) -> std::io::Result<()> {
        let sensor_discovery_clone = Arc::clone(self);
        let actuator_discovery_clone = Arc::clone(self);

        // Continuously attempt to discover new devices on the network
        std::thread::spawn(move || loop {
            {
                let mut ctrl = sensor_discovery_clone.lock().unwrap();
                ctrl.discover("_sensor").unwrap();
            }
            std::thread::sleep(Duration::from_secs(30));
        });

        std::thread::spawn(move || loop {
            {
                let mut ctrl = actuator_discovery_clone.lock().unwrap();
                ctrl.discover("_actuator").unwrap();
            }
            std::thread::sleep(Duration::from_secs(30));
        });

        // Cycle through and poll the Sensors, if the return Datum is outside a defined range
        // send a command off to the Actuator
        let self_api_clone = Arc::clone(self);
        std::thread::spawn(move || loop {
            // Create a temp vec to hold the data history as there is a lock on the controller and
            // we can't populate the history until the lock is released.
            let mut data_history: Vec<(Id, SensorHistory)> = Vec::new();
            {
                let ctrl = self_api_clone.lock().unwrap();
                for (id, addr) in ctrl.sensor_addresses.iter().clone() {
                    let trimmed_host = addr.host.trim_end_matches('.');
                    let url = format!("{}:{}", trimmed_host, addr.port);

                    if let Ok(datum) = ctrl.read_sensor(url.as_str()) {
                        // TODO What information do we want this to contain?
                        let history_id = Id::new(&Uuid::new_v4().to_string());
                        let sensor_history = SensorHistory {
                            id: history_id.clone(),
                            name: Name::new("sensor_data"),
                            data: vec![datum.clone()],
                        };

                        data_history.push((history_id, sensor_history));

                        // TODO We need a way to compare Datums
                        // TODO replace with actual min/max values for a given sensor
                        // TODO Remove fake_data, this is just so the next section gets called.
                        let fake_data = 5.0;
                        if fake_data < 10.0 {
                            // Get actuator address since Id is same for actuator/sensor pairings
                            let addr = ctrl.actuator_addresses.get(id).unwrap();
                            let trimmed_host = addr.host.trim_end_matches('.');
                            let url = format!("{}:{}", trimmed_host, addr.port);

                            // TODO Based on some logic we determine if we need the temp to go up or down
                            // Create an actuator command and attempt to deserialize it for http transfer.
                            let command = TemperatureActuatorCommand::SetMaxTemperature(100.0);
                            let command_json = serde_json::to_string(&command).unwrap();

                            ctrl.command_actuator(url.as_str(), command_json).unwrap()
                        }
                    }
                }
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

impl Controller {
    pub fn new() -> Controller {
        Controller::default()
    }

    /// Loops continually, attempting to discover new devices on the network.
    pub fn discover(&mut self, group: &str) -> std::io::Result<()> {
        let mdns = mdns_sd::ServiceDaemon::new().unwrap();

        let service_type = format!("{}._tcp.local.", group);
        let receiver = mdns.browse(service_type.as_str()).unwrap();

        while let Ok(event) = receiver.recv() {
            if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                let fullname = info.get_fullname();
                let group = info.get_type();
                let host = info.get_hostname();
                let port = info.get_port();

                println!(
                    "[discover] controller found: {} at {:?}:{}",
                    fullname, host, port
                );

                self.commit_to_memory(fullname, group, host, port);

                break;
            }
        }

        Ok(())
    }

    /// Adds the address of a `Device` to this `Controller`s memory.
    fn commit_to_memory(&mut self, fullname: &str, group: &str, host: &str, port: u16) {
        // hello_world.how_are.you => hello_world
        let full_id = fullname.split('.').next().unwrap_or_default();

        // hello_world => world
        let id_str = full_id.split('_').nth(1).unwrap_or_default();
        let id = Id::new(id_str);

        println!(
            "[commit_to_memory] fullname '{}' converted to id '{}'",
            fullname, id_str
        );

        let group = group.split('.').next();

        match group {
            Some("_sensor") => {
                let info = Address::new(host, port);
                self.sensor_addresses.insert(id.clone(), info);
            }
            Some("_actuator") => {
                let info = Address::new(host, port);
                self.actuator_addresses.insert(id.clone(), info);
            }
            _ => panic!(
                "[commit_to_memory] unknown group '{}' (expected '_sensor' or '_actuator')",
                group.unwrap()
            ),
        }
    }

    /// Connects to an address, sends the specified request, and returns the response
    fn send_request(address: &str, request: &str) -> String {
        println!("[send_request] connecting to url: {}", address);

        let mut stream = TcpStream::connect(address).unwrap();

        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        String::from(
            str::from_utf8(&response)
                .map(|s| s.trim())
                .unwrap_or("Failed to read response"),
        )
    }

    /// Attempts to get the latest `Datum` from the `Sensor` with the specified `Id`.
    pub fn read_sensor(&self, address: &str) -> Result<Datum, String> {
        // send the minimum possible payload. We basically just want to ping the Sensor
        // see: https://stackoverflow.com/a/9734866
        let request = "GET / HTTP/1.1\r\n\r\n";

        let response = Controller::send_request(address, request);

        println!(
            "[read_sensor] response from url {}:\n----------\n{}\n----------",
            address, response
        );

        // parse the response and return it
        Datum::parse(response.lines().last().unwrap_or_default())
    }

    pub fn command_actuator(&self, address: &str, command_json: String) -> std::io::Result<()> {
        let content_type = "application/json";
        let content_length = command_json.len();

        // Place the serialized command inside the POST payload
        let request = format!(
            "POST HTTP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type, content_length, command_json
        );

        let response = Controller::send_request(address, request.as_str());

        println!(
            "[command_actuator] response from url {}:\n----------\n{}\n----------",
            address, response
        );

        Ok(())
    }
}

/// The `SensorHistory` holds the unique `id`, the user-friendly `name`, and some history of
/// `data` points.
struct SensorHistory {
    #[allow(dead_code)] // remove this ASAP
    id: Id,
    #[allow(dead_code)] // remove this ASAP
    name: Name,
    #[allow(dead_code)] // remove this ASAP
    data: Vec<Datum>,
}

#[cfg(test)]
mod controller_tests {
    // use super::*;
    // TODO add back in once Controller api confirmed
    // #[test]
    // fn test_get_sensor_address() {
    //     let mut controller = Controller::new();
    //     let fullname = "hello_world.how_are.you";
    //     let id = Id::new("world");
    //
    //     controller.commit_to_memory(fullname, "_sensor", "localhost", 8080);
    //
    //     assert_eq!(
    //         controller.get_device_address(id),
    //         Ok(String::from("localhost:8080"))
    //     );
    // }
    //
    // #[test]
    // fn test_get_actuator_address() {
    //     let mut controller = Controller::new();
    //     let fullname = "hello_world.how_are.you";
    //     let id = Id::new("world");
    //
    //     controller.commit_to_memory(fullname, "_actuator", "localhost", 8080);
    //
    //     assert_eq!(
    //         controller.get_device_address(id),
    //         Ok(String::from("localhost:8080"))
    //     );
    // }
}
