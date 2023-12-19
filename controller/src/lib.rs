use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mdns::{discover, RecordKind};
use pin_utils::pin_mut;
use reqwest::Client;
use tokio::time;

use datum::Datum;

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so historic data
#[allow(dead_code)] // remove ASAP
struct Controller {
    /// Holds data queried from `Sensor`s
    data: HashMap<sensor::Id, SensorHistory>,
}

#[allow(dead_code)] // remove ASAP
/// The `SensorHistory` holds the unique `id`, the user-friendly `name`, and some history of
/// `data` points.
struct SensorHistory {
    id: sensor::Id,
    name: sensor::Name,
    data: Vec<Datum>,
}

// TODO add getters here as needed once we need to query the Controller's state for the HTML page

#[derive(Debug, Clone)]
struct Sensor {
    name: String,
    host: String,
    port: u16,
}

impl Default for Sensor {
    fn default() -> Sensor {
        Sensor {
            name: "".to_string(),
            host: "".to_string(),
            port: 0,
        }
    }
}

// TODO bring in SensorHistory
type Sensors = Arc<Mutex<HashMap<String, Sensor>>>;

pub async fn discover_sensors(sensors: Sensors) -> std::io::Result<()> {
    let service_type = "_sensor._tcp.local";

    loop {
        let service_stream = match discover::all(service_type, Duration::from_secs(60)) {
            Ok(discover) => discover.listen(),
            Err(e) => {
                eprintln!("error with discovery: {}", e);
                return Err(Error::new(ErrorKind::Other, "Discover error"));
            }
        };

        pin_mut!(service_stream);

        while let Some(result) = service_stream.next().await {
            match result {
                Ok(response) => {
                    let mut sensor = Sensor::default();
                    for record in response.records() {
                        match &record.kind {
                            RecordKind::A(ipv4) => {
                                sensor.host = ipv4.to_string();
                                sensor.port = response.port().unwrap();
                            }
                            RecordKind::PTR(full_domain) => {
                                sensor.name = full_domain.split(".").next().unwrap().to_string();
                            }
                            _ => {}
                        }
                    }

                    let mut sensors_map = sensors.lock().unwrap();
                    if !sensors_map.contains_key(&sensor.name) {
                        println!("Adding new Sensor: {:?}", sensor);
                        sensors_map.insert(sensor.name.clone(), sensor);
                    }
                }
                Err(e) => {
                    eprintln!("Error in streaming sensor service locations: {:?}", e);
                    break;
                }
            }
        }
    }
}

pub async fn query_sensors(sensors: Sensors) -> Result<(), Box<dyn std::error::Error + Send>> {
    let client = Client::default();

    loop {
        let current_sensors = {
            let lock = sensors.lock().unwrap();
            lock.clone()
        };

        for (name, sensor) in current_sensors.iter() {
            let url = format!("http://{}:{}/datum", sensor.host, sensor.port);

            let response = match client.get(&url).send().await {
                Ok(response) => response,
                Err(err) => {
                    eprintln!("Request error: {}", err);
                    continue;
                }
            };

            let status = response.status().clone();

            match response.text().await {
                Ok(body) => {
                    println!("Sensor {}: Data: {}", name, body);
                }
                Err(_) => {
                    eprintln!("Request to {} failed: {}", url, status);
                }
            }
        }

        time::sleep(Duration::from_secs(30)).await;
    }
}
