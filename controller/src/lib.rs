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
    contact_info: HashMap<Id, ContactInformation>
}

#[derive(Debug)]
struct ContactInformation {
    host: String,
    port: String
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
            contact_info: HashMap::new()
        }
    }
}

impl Controller {
    pub fn new() -> Controller {
        Controller::default()
    }

    /// Loops continually, attempting to discover new devices on the network.
    pub fn discover(&mut self, group: &str) -> std::io::Result<()>{
        let mdns = mdns_sd::ServiceDaemon::new().unwrap();

        let service_type = format!("{}._tcp.local.", group);
        let receiver = mdns.browse(service_type.as_str()).unwrap();

        while let Ok(event) = receiver.recv() {
            if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                println!("The controller has discovered: {} at {:?}:{}", info.get_fullname(),
                         info.get_addresses(), info.get_port());

                let full_id = info.get_fullname().split(".").next().unwrap_or_default();
                let id_str = full_id.split("_").nth(1).unwrap_or_default();

                let id = Id::new(id_str);

                match info.get_type().split(".").next().unwrap_or_default() {
                    "_sensor" => {
                        let sensor_info = ContactInformation {
                            host: info.get_hostname().to_string(),
                            port: info.get_port().to_string(),
                        };

                        if !self.contact_info.contains_key(&id) {
                            self.contact_info.insert(id.clone(), sensor_info);
                        }
                    },
                    "_actuator" => {
                        let actuator_info = ContactInformation {
                            host: info.get_hostname().to_string(),
                            port: info.get_port().to_string(),
                        };

                        if !self.contact_info.contains_key(&id) {
                            self.contact_info.insert(id.clone(), actuator_info);
                        }
                    }
                    _ => {}
                }
                break;
            }
        }

        Ok(())
    }

    pub fn read_sensor(&self, id: Id) -> std::io::Result<()> {
        // Get sensor from list
        println!("Reading Sensor Id: {}", id.0.clone());

        if !self.contact_info.contains_key(&id) {
            println!("Sensor Id not found in contact_info.");
            return Ok(());
        }

        let sensor = self.contact_info.get(&id).unwrap();

        let trimmed_host = sensor.host.trim_end_matches('.');
        let url = format!("{}:{}", trimmed_host, sensor.port);

        println!("Calling url {}", url.clone());

        let mut stream = TcpStream::connect(url).unwrap();
        let request = "GET / HTTP/1.1\r\n\r\n";

        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        println!("response: {}", str::from_utf8(&response).unwrap_or("Failed to read response"));

        Ok(())
    }

    pub fn command_actuator(&self, id: Id) -> std::io::Result<()> {
        println!("Commanding Actuator Id: {}", id.0.clone());

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

        println!("response: {}", str::from_utf8(&response).unwrap_or("Failed to read response"));

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
