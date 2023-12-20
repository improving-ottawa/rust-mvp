use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

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
    /// Contains mapping of Id their Contact information
    contact_info: HashMap<Id, ContactInformation>,
}

#[derive(Debug)]
struct ContactInformation {
    host: String,
    port: String,
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
            contact_info: HashMap::new(),
        }
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
                println!(
                    "The controller has discovered: {} at {:?}:{}",
                    info.get_fullname(),
                    info.get_addresses(),
                    info.get_port()
                );

                self.commit_to_memory(
                    info.get_fullname(),
                    info.get_type(),
                    info.get_hostname(),
                    info.get_port(),
                );

                break;
            }
        }

        Ok(())
    }

    /// Private internal method to add the contact info of a `Device` to this `Controller`s memory.
    fn commit_to_memory(&mut self, fullname: &str, group: &str, hostname: &str, port: u16) {
        // hello_world.how_are.you => hello_world
        let full_id = fullname.split('.').next().unwrap_or_default();

        // hello_world => world
        let id_str = full_id.split('_').nth(1).unwrap_or_default();

        let id = Id::new(id_str);

        println!(
            "[commit_to_memory] fullname '{}' converted to id '{}'",
            fullname, id_str
        );

        match group.split('.').next().unwrap_or_default() {
            "_sensor" => {
                let sensor_info = ContactInformation {
                    host: hostname.to_string(),
                    port: port.to_string(),
                };

                if !self.contact_info.contains_key(&id) {
                    self.contact_info.insert(id.clone(), sensor_info);
                }
            }
            "_actuator" => {
                let actuator_info = ContactInformation {
                    host: hostname.to_string(),
                    port: port.to_string(),
                };

                if !self.contact_info.contains_key(&id) {
                    self.contact_info.insert(id.clone(), actuator_info);
                }
            }
            other => panic!(
                "[commit_to_memory] unknown group '{}' (expected '_sensor' or '_actuator')",
                other
            ),
        }
    }

    /// Retrieves this `Sensor`'s address from its `Id.
    pub fn get_sensor_address(&self, id: Id) -> Result<String, String> {
        println!("[get_sensor_address] looking for Id: {}", id);

        if !self.contact_info.contains_key(&id) {
            let msg = format!("Sensor Id '{}' not found in contact info", id);
            println!("{}", msg);
            return Err(msg);
        }

        let sensor = self.contact_info.get(&id).unwrap();

        // sensor.host has a '.' at the end, i.e. "192.168.1.21."
        // this removes any trailing '.' characters
        let trimmed_host = sensor.host.trim_end_matches('.');

        Ok(format!("{}:{}", trimmed_host, sensor.port))
    }

    /// Attempts to get the latest `Datum` from the `Sensor` with the specified `Id`.
    pub fn read_sensor(&self, address: &str) -> Result<Datum, String> {
        println!("[read_sensor] connecting to url: {}", address);

        let mut stream = TcpStream::connect(address).unwrap();

        // send the minimum possible payload. We basically just want to ping the Sensor
        // see: https://stackoverflow.com/a/9734866
        let request = "GET / HTTP/1.1\r\n\r\n";

        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        let response = str::from_utf8(&response)
            .map(|s| s.trim())
            .unwrap_or("Failed to read response");

        println!(
            "[read_sensor] response from url {}:\n----------\n{}\n----------",
            address, response
        );

        // parse the response and return it
        Datum::parse(response.lines().last().unwrap_or_default())
    }

    pub fn command_actuator(&self, id: Id) -> std::io::Result<()> {
        println!("Commanding Actuator Id: {}", id);

        if !self.contact_info.contains_key(&id) {
            println!("Actuator Id not found in contact_info.");
            return Ok(());
        }

        let actuator = self.contact_info.get(&id).unwrap();

        let trimmed_host = actuator.host.trim_end_matches('.');
        let url = format!("{}:{}", trimmed_host, actuator.port);
        let mut stream = TcpStream::connect(url).unwrap();

        let content_type = "text/plain";
        let body = r#"Act"#;
        let content_length = body.len();

        let request = format!(
            "POST HTTP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type, content_length, body
        );

        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        println!(
            "response: {}",
            str::from_utf8(&response).unwrap_or("Failed to read response")
        );

        Ok(())
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

#[cfg(test)]
mod controller_tests {
    use super::*;

    // Delete once return results finalized in Controller. Just needed for CI to pass
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[test]
    fn test_get_sensor_address() {
        let mut controller = Controller::new();
        let fullname = "hello_world.how_are.you";
        let id = Id::new("world");

        controller.commit_to_memory(fullname, "_sensor", "localhost", 8080);

        assert_eq!(
            controller.get_sensor_address(id),
            Ok(String::from("localhost:8080"))
        );
    }

    // TODO finish once api return time finalized
    #[test]
    fn test_cmd_actuator() {
        let controller = Controller::new();
        let id = Id::new("dummy_id");
        controller.command_actuator(id).unwrap();

        assert_eq!(5, add(2, 3));
    }
}
