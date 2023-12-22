use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use datum::Datum;
use device::Id;

pub struct State {
    // histories: HashMap<Id, SensorHistory>,
    sensors: Arc<Mutex<HashMap<Id, Address>>>,
    actuators: Arc<Mutex<HashMap<Id, Address>>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            // histories: HashMap::new(),
            sensors: Arc::new(Mutex::new(HashMap::new())),
            actuators: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn discover_sensors(&self) -> JoinHandle<()> {
        // clone the Arc<Mutex<>> around the sensors so we can update them in multiple threads
        let sensors = Arc::clone(&self.sensors);

        std::thread::spawn(move || {
            let mdns = mdns_sd::ServiceDaemon::new().unwrap();
            let service_type = "_sensor._tcp.local.";
            let receiver = mdns.browse(service_type).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    let fullname = info.get_fullname();
                    let host = info.get_hostname();
                    let port = info.get_port();

                    println!(
                        "[discover] controller found: {} at {:?}:{}",
                        fullname, host, port
                    );

                    // e.g. hello_world.how_are.you => hello_world
                    let full_id = fullname.split('.').next().unwrap_or_default();

                    // hello_world => world
                    let id_str = full_id.split('_').nth(1).unwrap_or_default();
                    let id = Id::new(id_str);

                    // whenever we find something new on the network, remember it
                    let sensors = sensors.lock();
                    let info = Address::new(host, port);
                    sensors.unwrap().insert(id, info);
                }
            }
        })
    }

    pub fn discover_actuators(&self) -> JoinHandle<()> {
        // clone the Arc<Mutex<>> around the actuators so we can update them in multiple threads
        let actuators = Arc::clone(&self.actuators);

        std::thread::spawn(move || {
            let mdns = mdns_sd::ServiceDaemon::new().unwrap();
            let service_type = "_actuator._tcp.local.";
            let receiver = mdns.browse(service_type).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    let fullname = info.get_fullname();
                    let host = info.get_hostname();
                    let port = info.get_port();

                    println!(
                        "[discover] controller found: {} at {:?}:{}",
                        fullname, host, port
                    );

                    // e.g. hello_world.how_are.you => hello_world
                    let full_id = fullname.split('.').next().unwrap_or_default();

                    // hello_world => world
                    let id_str = full_id.split('_').nth(1).unwrap_or_default();
                    let id = Id::new(id_str);

                    // whenever we find something new on the network, remember it
                    let actuators = actuators.lock();
                    let info = Address::new(host, port);
                    actuators.unwrap().insert(id, info);
                }
            }
        })
    }

    /// Connects to an address, sends the specified request, and returns the response
    #[allow(dead_code)] // remove this ASAP
    fn send_request(address: &str, request: &str) -> String {
        println!("[send_request] connecting to url: {}", address);

        let mut stream = TcpStream::connect(address).unwrap();

        stream.write_all(request.as_bytes()).unwrap();

        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();

        String::from(
            std::str::from_utf8(&response)
                .map(|s| s.trim())
                .unwrap_or("Failed to read response"),
        )
    }

    /// Attempts to get the latest `Datum` from the `Sensor` with the specified `Id`.
    #[allow(dead_code)] // remove this ASAP
    pub fn read_sensor(&self, address: &str) -> Result<Datum, String> {
        // send the minimum possible payload. We basically just want to ping the Sensor
        // see: https://stackoverflow.com/a/9734866
        let request = "GET / HTTP/1.1\r\n\r\n";

        let response = State::send_request(address, request);

        println!(
            "[read_sensor] response from url {}:\n----------\n{}\n----------",
            address, response
        );

        // parse the response and return it
        Datum::parse(response.lines().last().unwrap_or_default())
    }

    #[allow(dead_code)] // remove this ASAP
    pub fn command_actuator(&self, address: &str, command_json: String) -> std::io::Result<()> {
        let content_type = "application/json";
        let content_length = command_json.len();

        // Place the serialized command inside the POST payload
        let request = format!(
            "POST HTTP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type, content_length, command_json
        );

        let response = State::send_request(address, request.as_str());

        println!(
            "[command_actuator] response from url {}:\n----------\n{}\n----------",
            address, response
        );

        Ok(())
    }

    // pub fn poll(&self) {
    //     for (id, addr) in self.sensors.iter().clone() {
    //
    //         let trimmed_host = addr.host.trim_end_matches('.');
    //         let url = format!("{}:{}", trimmed_host, addr.port);
    //
    //         if let Ok(datum) = self.read_sensor(url.as_str()) {
    //
    //             // get existing history, if there is any
    //             let thing: Option<SensorHistory> = self.histories.get(id).map(|b| b.clone())
    //
    //             let sensor_history = SensorHistory {
    //                 id: id.clone(),
    //                 data: vec![datum.clone()],
    //             };
    //
    //             self.histories.push((history_id, sensor_history));
    //
    //             // TODO We need a way to compare Datums
    //             // TODO replace with actual min/max values for a given sensor
    //             // TODO Remove fake_data, this is just so the next section gets called.
    //             let fake_data = 5.0;
    //             if fake_data < 10.0 {
    //                 // Get actuator address since Id is same for actuator/sensor pairings
    //                 let addr = ctrl.actuators.get(id).unwrap();
    //                 let trimmed_host = addr.host.trim_end_matches('.');
    //                 let url = format!("{}:{}", trimmed_host, addr.port);
    //
    //                 // TODO Based on some logic we determine if we need the temp to go up or down
    //                 // Create an actuator command and attempt to deserialize it for http transfer.
    //                 let command = TemperatureActuatorCommand::SetMaxTemperature(100.0);
    //                 let command_json = serde_json::to_string(&command).unwrap();
    //
    //                 ctrl.command_actuator(url.as_str(), command_json).unwrap()
    //             }
    //         }
    //     }
    // }
}

#[derive(Debug)]
struct Address {
    #[allow(dead_code)] // remove ASAP
    host: String,
    #[allow(dead_code)] // remove ASAP
    port: String,
}

impl Address {
    fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port: port.to_string(),
        }
    }
}

#[allow(dead_code)] // remove ASAP
struct SensorHistory {
    id: Id,
    data: Vec<Datum>,
}
