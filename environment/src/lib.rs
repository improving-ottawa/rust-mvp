use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rand::Rng;

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
        // FIXME are we worried about deadlocks here?
        let mut attributes = self.attributes.lock().unwrap();
        attributes.insert(id, value);
    }

    fn get(&self, id: &Id) -> Datum {
        // FIXME are we worried about deadlocks here?
        let mut attributes = self.attributes.lock().unwrap();
        match attributes.get(id) {
            Some(datum) => datum.clone(),
            None => {
                // FIXME we need to return the type (bool, f32, i32) of data the Sensor expects
                let random_unit = Some(DatumUnit::Unitless);
                let random_datum = Environment::generate_random_datum(random_unit);
                attributes.insert(id.clone(), random_datum.clone()); // Insert the random Datum into attributes
                random_datum
            }
        }
    }

    fn generate_random_datum(unit: Option<DatumUnit>) -> Datum {
        match rand::thread_rng().gen_range(0..3) {
            0 => Environment::generate_random_bool(unit),
            1 => Environment::generate_random_f32(unit),
            _ => Environment::generate_random_i32(unit),
        }
    }

    fn generate_random_bool(unit: Option<DatumUnit>) -> Datum {
        let mut rng = rand::thread_rng();
        let value = DatumValue::Bool(rng.gen());
        Datum::new(value, unit, Utc::now())
    }

    fn generate_random_f32(unit: Option<DatumUnit>) -> Datum {
        let mut rng = rand::thread_rng();
        let value = DatumValue::Float(rng.gen());
        Datum::new(value, unit, Utc::now())
    }

    fn generate_random_i32(unit: Option<DatumUnit>) -> Datum {
        let mut rng = rand::thread_rng();
        let value = DatumValue::Int(rng.gen());
        Datum::new(value, unit, Utc::now())
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
        let mut request = Vec::new();
        stream.read_to_end(&mut request).unwrap();

        let request = std::str::from_utf8(&request)
            .map(|s| s.trim())
            .unwrap_or("Failed to read response");

        let response = self.handle_request(request);

        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    fn extract_command(&self, _request: &str) -> (Id, String) {
        todo!()
    }

    fn execute_command(&self, id: &Id, _command: &str) -> Option<Datum> {
        // TODO: Implement actual command execution logic
        // Maybe the command should be an struct or enum with a type and a value?

        let mut attributes = self.attributes.lock().unwrap();

        attributes.get_mut(id).map(|datum| datum.clone())
    }

    fn extract_id(&self, request: &str) -> Option<Id> {
        let parts: Vec<&str> = request.split('/').collect();
        if parts.len() > 2 {
            Some(Id::new(parts[2]))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod env_tests {
    use chrono::Utc;
    use regex::Regex;

    use datum::DatumUnit;

    use super::*;

    #[test]
    fn test_set_and_get_datum() {
        let environment = Environment::new();
        let id = Id::new("test_id");

        let actual = Datum::new(24, Some(DatumUnit::Unitless), Utc::now());
        environment.set(id.clone(), actual.clone());
        let expected = environment.get(&id);

        assert_eq!(actual.value, expected.value);
        assert_eq!(actual.unit, expected.unit);
        assert_eq!(actual.timestamp, expected.timestamp);
    }

    #[test]
    fn test_handle_get_request() {
        let environment = Environment::new();

        fn contains_datum(response: String) -> bool {
            let datum_regex =
                Regex::new(r"Datum \{ value: .*, unit: .*, timestamp: .* \}").unwrap();
            datum_regex.is_match(&response)
        }

        let get_request = "GET /get/test_id";
        let get_response = environment.handle_request(get_request);

        assert!(contains_datum(get_response));
    }

    #[test]
    fn test_handle_get_request_undefined() {
        let environment = Environment::new();

        let undefined_request = "GET /undefined";
        let undefined_response = environment.handle_request(undefined_request);
        assert_eq!(undefined_response, "HTTP/1.1 404 Not Found\r\n\r\n");
    }
}
