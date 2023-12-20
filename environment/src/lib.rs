use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use datum::{Datum, DatumUnit, DatumValue, DatumValueType};
use device::Id;

/// A test-only example environment which produces data detected by `Sensor`s.
///
/// The `Environment` can be mutated by `Actuator`s.
#[derive(Default)] // gives us an "empty" Environment with Environment::default()
struct Environment<'a> {
    attributes: Arc<Mutex<HashMap<Id, DatumGenerator<'a>>>>,
    #[allow(dead_code)] // remove ASAP
    created_at: DateTime<Utc>,
}

struct DatumGenerator<'a> {
    generator: Box<dyn FnMut(DateTime<Utc>) -> DatumValue + 'a>,
    unit: DatumUnit,
}

// DatumGenerator must implement Sync and Send in order to be wrapped in an Arc<Mutex<...>>, above
unsafe impl Sync for DatumGenerator<'_> {}
unsafe impl Send for DatumGenerator<'_> {}

impl<'a> DatumGenerator<'a> {
    fn new(
        generator: Box<impl FnMut(DateTime<Utc>) -> DatumValue + 'a>,
        unit: DatumUnit,
    ) -> DatumGenerator<'a> {
        DatumGenerator { generator, unit }
    }

    fn generate(&mut self) -> Datum {
        let now = Utc::now();
        let value = (*self.generator)(now);
        Datum::new(value, self.unit, now)
    }
}

#[allow(dead_code)] // remove ASAP
impl<'a> Environment<'a> {
    fn new() -> Environment<'a> {
        Environment {
            attributes: Arc::new(Mutex::new(HashMap::new())),
            created_at: Utc::now(),
        }
    }

    fn set(&self, id: Id, generator: DatumGenerator<'a>) {
        // FIXME are we worried about deadlocks here?
        let mut attributes = self.attributes.lock().unwrap();
        attributes.insert(id, generator);
    }

    fn generator_linearly_increasing_f32(
        &self,
        slope: f32,
        noise: f32,
        unit: DatumUnit,
    ) -> DatumGenerator<'a> {
        let t0 = self.created_at.timestamp_millis();
        let mut rng = thread_rng();

        let f = move |time: DateTime<Utc>| -> DatumValue {
            let delta = time.timestamp_millis() - t0;
            let noise_factor = rng.gen_range(-1.0..1.0);
            DatumValue::Float((delta as f32) * slope + noise * noise_factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }

    fn generator_linearly_decreasing_f32(
        &self,
        slope: f32,
        noise: f32,
        unit: DatumUnit,
    ) -> DatumGenerator<'a> {
        let t0 = self.created_at.timestamp_millis();
        let mut rng = thread_rng();

        let f = move |time: DateTime<Utc>| -> DatumValue {
            let delta = time.timestamp_millis() - t0;
            let noise_factor = rng.gen_range(-1.0..1.0);
            DatumValue::Float((delta as f32) * slope + noise * noise_factor * -1.0)
        };

        DatumGenerator::new(Box::new(f), unit)
    }

    fn get(&mut self, id: &Id, kind: DatumValueType, unit: DatumUnit) -> Datum {
        // FIXME are we worried about deadlocks here?
        let mut attributes = self.attributes.lock().unwrap();
        match attributes.get_mut(id) {
            Some(generator) => generator.generate(),
            None => {
                // we need to return the type (bool, f32, i32) of data the Sensor expects

                let mut rng = thread_rng();

                let generator = match kind {
                    DatumValueType::Bool => todo!(),
                    DatumValueType::Int => todo!(),
                    DatumValueType::Float => {
                        let slope = rng.gen_range(0.01..0.10); // arbitrarily selected range of slopes
                        let noise = rng.gen_range(0.01..0.10); // arbitrary selected range of noise values

                        match rng.gen_range(0..1) {
                            0 => self.generator_linearly_increasing_f32(slope, noise, unit),
                            _ => self.generator_linearly_decreasing_f32(slope, noise, unit),
                        }
                    }
                };

                // register this Datum generator to this Id
                attributes.insert(id.clone(), generator);

                // generate a random value
                attributes.get_mut(id).unwrap().generate()
            }
        }
    }

    fn generate_random_datum(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        match rand::thread_rng().gen_range(0..3) {
            0 => Environment::generate_random_bool(rng, unit),
            1 => Environment::generate_random_f32(rng, unit),
            _ => Environment::generate_random_i32(rng, unit),
        }
    }

    fn generate_random_bool(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        Datum::new_now(DatumValue::Bool(rng.gen()), unit)
    }

    fn generate_random_f32(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        Datum::new_now(DatumValue::Float(rng.gen()), unit)
    }

    fn generate_random_i32(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        Datum::new_now(DatumValue::Int(rng.gen()), unit)
    }

    pub fn handle_request(&mut self, request: &str) -> String {
        if request.starts_with("POST /set/") {
            // if the Environment gets a command from an actuator with a Device::Id that it is not
            // yet aware of, it should ignore it

            // Not complete, but this is the general idea
            // Extract ID and command
            let (id, command) = self.extract_command(request);
            match self.execute_command(&id, &command) {
                Some(datum) => format!("HTTP/1.1 200 OK\r\n\r\n{:?}", datum),
                None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
            }
        } else if request.starts_with("GET /get/") {
            // if the Environment gets a request from a sensor with a Device::Id that it is not
            // yet aware of, it should save the Id and pick from a random data generator

            let parsed = Environment::parse_get_request(request);

            if let Ok((id, value_type, unit)) = parsed {
                let datum = self.get(&id, value_type, unit);
                format!("HTTP/1.1 200 OK\r\n\r\n{:?}", datum)
            } else {
                let msg = parsed.unwrap_err();
                format!("HTTP/1.1 400 Bad Request\r\n\r\n{}", msg)
            }
        } else {
            "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
        }
    }

    fn start_server(&mut self) -> std::io::Result<()> {
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

    fn handle_client(&mut self, mut stream: TcpStream) -> std::io::Result<()> {
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

    fn execute_command(&self, _id: &Id, _command: &str) -> Option<Datum> {
        // TODO: Implement actual command execution logic
        // Maybe the command should be an struct or enum with a type and a value?
        todo!()
    }

    fn parse_get_request(request: &str) -> Result<(Id, DatumValueType, DatumUnit), String> {
        // example request: "GET /get/test_id/float/°C"
        let mut parts = request.split('/');

        parts.next(); // throw out "GET"
        parts.next(); // throw out "get"

        match (parts.next(), parts.next(), parts.next()) {
            (Some(id), Some(value_type), Some(unit)) => {
                match (DatumValueType::parse(value_type), DatumUnit::parse(unit)) {
                    (Ok(value_type), Ok(unit)) => Ok((Id::new(id), value_type, unit)),
                    (Ok(_), Err(msg)) => Err(msg),
                    (Err(msg), Ok(_)) => Err(msg),
                    (Err(msg1), Err(msg2)) => Err(format!("{}\n{}", msg1, msg2)),
                }
            }
            _ => Err(format!(
                "Cannot split {} into /get/<sensor_id>/<type>/<unit>",
                request
            )),
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
        let mut environment = Environment::new();

        let id = Id::new("test_id");
        let value_type = DatumValueType::Int;
        let unit = DatumUnit::Unitless;

        let constant = |_: DateTime<Utc>| -> DatumValue { DatumValue::Int(42) };

        let generator = DatumGenerator::new(Box::new(constant), unit);

        environment.set(id.clone(), generator);
        let datum = environment.get(&id, value_type, unit);

        assert_eq!(datum.value, DatumValue::Int(42));
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_handle_get_request() {
        let mut environment = Environment::new();

        fn contains_datum(response: String) -> bool {
            let datum_regex =
                Regex::new(r"Datum \{ value: .*, unit: .*, timestamp: .* \}").unwrap();
            datum_regex.is_match(&response)
        }

        let get_request = "GET /get/test_id/float/°C";
        let get_response = environment.handle_request(get_request);

        println!("response: {}", get_response);

        assert!(contains_datum(get_response));
    }

    #[test]
    fn test_handle_get_request_undefined() {
        let mut environment = Environment::new();

        let undefined_request = "GET /undefined";
        let undefined_response = environment.handle_request(undefined_request);
        assert_eq!(undefined_response, "HTTP/1.1 404 Not Found\r\n\r\n");
    }
}
