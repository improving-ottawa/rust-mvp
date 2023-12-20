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
    /// Maps `Device` `Id`s to their contact information
    contact_info: HashMap<Id, ContactInformation>,
}

#[derive(Debug)]
struct ContactInformation {
    host: String,
    port: String,
}

impl ContactInformation {
    fn new(host: &str, port: u16) -> ContactInformation {
        ContactInformation {
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
                let fullname = info.get_fullname();
                let group = info.get_type();
                let host = info.get_hostname();
                let port = info.get_port();

                println!(
                    "[discover] controller found: {} at {:?}:{}",
                    fullname, host, port
                );

                self.commit_to_memory(fullname, group, host, port);

                break; // FIXME why is this here? Doesn't this kill the discovery process?
            }
        }

        Ok(())
    }

    /// Adds the contact info of a `Device` to this `Controller`s memory.
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

        let group = group.split('.').next().unwrap_or_default();

        if group == "_sensor" || group == "_actuator" {
            let info = ContactInformation::new(hostname, port);
            self.contact_info.insert(id.clone(), info);
        } else {
            panic!(
                "[commit_to_memory] unknown group '{}' (expected '_sensor' or '_actuator')",
                group
            )
        }
    }

    /// Retrieves this `Device`'s address from its `Id`.
    pub fn get_device_address(&self, id: Id) -> Result<String, String> {
        println!("[get_device_address] looking for Id: {}", id);

        if !self.contact_info.contains_key(&id) {
            let msg = format!("Device Id '{}' not found in contact info", id);
            println!("[get_device_address] {}", msg);
            return Err(msg);
        }

        let device = self.contact_info.get(&id).unwrap();

        // sensor.host has a '.' at the end, i.e. "192.168.1.21."
        // this removes any trailing '.' characters
        let trimmed_host = device.host.trim_end_matches('.');

        Ok(format!("{}:{}", trimmed_host, device.port))
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
    pub fn read_sensor(address: &str) -> Result<Datum, String> {
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

    pub fn command_actuator(address: &str) -> std::io::Result<()> {
        let content_type = "text/plain";
        let body = r#"Act"#;
        let content_length = body.len();

        let request = format!(
            "POST HTTP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type, content_length, body
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
#[allow(dead_code)] // remove ASAP
struct SensorHistory {
    id: Id,
    name: Name,
    data: Vec<Datum>,
}

#[cfg(test)]
mod controller_tests {
    use super::*;

    #[test]
    fn test_get_sensor_address() {
        let mut controller = Controller::new();
        let fullname = "hello_world.how_are.you";
        let id = Id::new("world");

        controller.commit_to_memory(fullname, "_sensor", "localhost", 8080);

        assert_eq!(
            controller.get_device_address(id),
            Ok(String::from("localhost:8080"))
        );
    }

    #[test]
    fn test_get_actuator_address() {
        let mut controller = Controller::new();
        let fullname = "hello_world.how_are.you";
        let id = Id::new("world");

        controller.commit_to_memory(fullname, "_actuator", "localhost", 8080);

        assert_eq!(
            controller.get_device_address(id),
            Ok(String::from("localhost:8080"))
        );
    }
}
