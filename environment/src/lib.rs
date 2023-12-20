use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use datum::{Datum, DatumUnit, DatumValue};
use device::Id;

/// A test-only example environment which produces data detected by `Sensor`s.
///
/// The `Environment` can be mutated by `Actuator`s.
#[derive(Default)] // gives us an "empty" Environment with Environment::default()
struct Environment {
    attributes: Arc<Mutex<HashMap<Id, Datum>>>,
}

#[allow(dead_code)] // remove ASAP
impl Environment {
    fn new() -> Environment {
        Environment {
            attributes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn set(&self, id: Id, value: Datum) {
        let mut attributes = self.attributes.lock().unwrap();
        attributes.insert(id, value);
    }

    fn get(&self, id: &Id) -> Datum {
        let mut attributes = self.attributes.lock().unwrap();
        match attributes.get(id) {
            Some(datum) => datum.clone(),
            None => {
                let random_datum = self.generate_random_datum();
                attributes.insert(id.clone(), random_datum.clone()); // Insert the random Datum into attributes
                random_datum
            }
        }
    }

    fn generate_random_datum(&self) -> Datum {
        let mut rng = rand::thread_rng();
        // generate a random datum
        Datum {
            value: match rng.gen_range(0..3) {
                0 => DatumValue::Bool(rng.gen()),
                1 => DatumValue::Float(rng.gen()),
                2 => DatumValue::Int(rng.gen()),
                _ => DatumValue::Int(0),
            },
            unit: Some(DatumUnit::Unitless), // or generate a random unit if needed
            timestamp: Utc::now(),
        }
    }

    pub fn handle_request(&self, request: &str) -> String {
        if request.starts_with("POST /set/") {
            // Not complete, but this is the general idea
            // Extract ID and command
            let (id, command) = self.extract_command(request);
            match self.execute_command(&id, &command) {
                Some(datum) => format!("HTTP/1.1 200 OK\r\n\r\n{:?}", datum),
                None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
            }
        } else if request.starts_with("GET /get/") {
            // Extract ID
            let id = self.extract_id(request).unwrap();
            let datum = self.get(&id);
            format!("HTTP/1.1 200 OK\r\n\r\n{:?}", datum)
        } else {
            "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
        }
    }

    fn start_server(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080")?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handle_client(stream)?;
                }
                Err(e) => eprintln!("Failed to handle client: {}", e),
            }
        }
        Ok(())
    }

    fn handle_client(&self, mut stream: TcpStream) -> std::io::Result<()> {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;
        let request = std::str::from_utf8(&buffer).unwrap();

        let response = self.handle_request(request);

        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    fn extract_command(&self, _request: &str) -> (Id, String) {
        // TODO: Implement actual command extraction from the request
        // This is a placeholder implementation
        (Id::new("_id"), "command".to_string());
        todo!("Implement command extraction")
    }

    fn execute_command(&self, id: &Id, _command: &str) -> Option<Datum> {
        // TODO: Implement actual command execution logic
        // Maybe the command should be an struct or enum with a type and a value?

        let mut attributes = self.attributes.lock().unwrap();
        if let Some(datum) = attributes.get_mut(id) {
            Some(datum.clone())
        } else {
            None
        }
    }

    fn extract_id(&self, request: &str) -> Option<Id> {
        let parts: Vec<&str> = request.split('/').collect();
        if parts.len() > 2 {
            Some(Id::new(parts[2]))
        } else {
            None
        }
    }

    // TODO add random data generation as necessary
}

#[cfg(test)]
mod env_tests {
    use super::*;
    use chrono::Utc;
    use datum::{DatumUnit, DatumValue};
    use regex::Regex;

    #[test]
    fn test_set_and_get_datum() {
        let environment = Environment::new();
        let id = Id::new("test_id");
        let datum = Datum {
            value: DatumValue::Int(42),
            unit: Some(DatumUnit::Unitless),
            timestamp: Utc::now(),
        };

        // Test set
        environment.set(id.clone(), datum.clone());
        // Test get
        let retrieved = environment.get(&id);
        assert_eq!(retrieved.value, DatumValue::Int(42));
        assert_eq!(retrieved.unit, Some(DatumUnit::Unitless));
    }

    #[test]
    fn test_handle_request() {
        let environment = Environment::new();
        let id = Id::new("test_id");
        let date_str = "2023-01-01T00:00:00Z";
        let specific_datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(date_str)
            .expect("Invalid date format")
            .with_timezone(&Utc);

        let test_datum = Datum {
            value: DatumValue::Int(42),
            unit: Some(DatumUnit::Unitless),
            timestamp: specific_datetime,
        };

        // Simulating a GET request

        fn contains_datum(response: String) -> bool {
            let datum_regex = Regex::new(r"Datum").unwrap();
            datum_regex.is_match(&response)
        }

        let get_request = "GET /get/test_id";
        let get_response = environment.handle_request(get_request);

        assert!(contains_datum(get_response));

        // Simulating a request to an undefined route
        let undefined_request = "GET /undefined";
        let undefined_response = environment.handle_request(undefined_request);
        assert_eq!(undefined_response, "HTTP/1.1 404 Not Found\r\n\r\n");
    }
}
