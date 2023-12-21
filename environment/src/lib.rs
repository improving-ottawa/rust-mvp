use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;

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
    #[allow(dead_code)] // remove this ASAP
    attributes: Mutex<HashMap<Id, DatumGenerator<'a>>>,
}

struct DatumGenerator<'a> {
    #[allow(dead_code)] // remove this ASAP
    generator: Mutex<Box<dyn FnMut(DateTime<Utc>) -> DatumValue + 'a>>,
    #[allow(dead_code)] // remove this ASAP
    unit: DatumUnit,
}

impl<'a> DatumGenerator<'a> {
    #[allow(dead_code)] // remove this ASAP
    fn new(
        generator: Box<dyn FnMut(DateTime<Utc>) -> DatumValue + 'a>,
        unit: DatumUnit,
    ) -> DatumGenerator<'a> {
        DatumGenerator {
            generator: Mutex::new(generator),
            unit,
        }
    }

    #[allow(dead_code)] // remove this ASAP
    fn generate(&self) -> Datum {
        let now = Utc::now();
        let mut generator = self.generator.lock().unwrap();
        let value = (*generator)(now);
        Datum::new(value, self.unit, now)
    }
}

impl<'a> Environment<'a> {
    #[allow(dead_code)] // remove this ASAP
    fn new() -> Environment<'a> {
        Environment {
            attributes: Mutex::new(HashMap::new()),
        }
    }

    #[allow(dead_code)] // remove this ASAP
    fn set(&self, id: Id, generator: DatumGenerator<'a>) {
        let mut attributes = self.attributes.lock().unwrap();
        attributes.insert(id, generator);
    }

    #[allow(dead_code)] // remove this ASAP
    fn generator_linearly_increasing_f32(
        &self,
        slope: f32,
        noise: f32,
        unit: DatumUnit,
    ) -> DatumGenerator<'a> {
        let starting_time = Utc::now().timestamp_millis(); // Fixed starting time
        let mut rng = thread_rng();

        let f = move |now: DateTime<Utc>| -> DatumValue {
            let delta = now.timestamp_millis() - starting_time;
            let noise_factor = rng.gen_range(-1.0..1.0) * noise;
            DatumValue::Float(delta as f32 * slope + noise_factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }

    #[allow(dead_code)] // remove this ASAP
    fn generator_linearly_decreasing_f32(
        &self,
        slope: f32,
        noise: f32,
        unit: DatumUnit,
    ) -> DatumGenerator<'a> {
        let starting_time = Utc::now().timestamp_millis(); // Fix the starting time
        let mut rng = thread_rng();

        let f = move |now: DateTime<Utc>| -> DatumValue {
            let delta = now.timestamp_millis() - starting_time;
            let noise_factor = rng.gen_range(-1.0..1.0);
            DatumValue::Float(-1.0 * (delta as f32) * slope + noise * noise_factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }

    #[allow(dead_code)] // remove this ASAP
    fn get(&mut self, id: &Id, kind: DatumValueType, unit: DatumUnit) -> Datum {
        let mut attributes = self.attributes.lock().unwrap();
        match attributes.get_mut(id) {
            Some(generator) => generator.generate(),
            None => {
                // we need to return the type (bool, f32, i32) of data the Sensor expects
                let mut rng = thread_rng();
                let generator = match kind {
                    DatumValueType::Bool => {
                        let mut last_value = false;
                        let f = move |_| -> DatumValue {
                            last_value = !last_value;
                            DatumValue::Bool(last_value)
                        };
                        DatumGenerator::new(Box::new(f), unit)
                    }
                    DatumValueType::Int => {
                        let slope = rng.gen_range(1..10);
                        let t0 = Utc::now().timestamp();
                        let f = move |now: DateTime<Utc>| -> DatumValue {
                            let delta = (now.timestamp() - t0) / 1000;
                            let increase = rng.gen_bool(0.5);
                            if increase {
                                DatumValue::Int((delta * slope) as i32)
                            } else {
                                DatumValue::Int((-delta * slope) as i32)
                            }
                        };
                        DatumGenerator::new(Box::new(f), unit)
                    }

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

    #[allow(dead_code)] // remove this ASAP
    fn generate_random_datum(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        match rand::thread_rng().gen_range(0..3) {
            0 => Environment::generate_random_bool(rng, unit),
            1 => Environment::generate_random_f32(rng, unit),
            _ => Environment::generate_random_i32(rng, unit),
        }
    }

    #[allow(dead_code)] // remove this ASAP
    fn generate_random_bool(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        Datum::new_now(DatumValue::Bool(rng.gen()), unit)
    }

    #[allow(dead_code)] // remove this ASAP
    fn generate_random_f32(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        Datum::new_now(DatumValue::Float(rng.gen()), unit)
    }

    #[allow(dead_code)] // remove this ASAP
    fn generate_random_i32(rng: &mut ThreadRng, unit: DatumUnit) -> Datum {
        Datum::new_now(DatumValue::Int(rng.gen()), unit)
    }

    #[allow(dead_code)] // remove this ASAP
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

    #[allow(dead_code)] // remove this ASAP
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

    #[allow(dead_code)] // remove this ASAP
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

    #[allow(dead_code)] // remove this ASAP
    fn extract_command(&self, _request: &str) -> (Id, String) {
        todo!()
    }

    #[allow(dead_code)] // remove this ASAP
    fn execute_command(&self, _id: &Id, _command: &str) -> Option<Datum> {
        // TODO: Implement actual command execution logic
        // Maybe the command should be an struct or enum with a type and a value?
        todo!()
    }

    #[allow(dead_code)] // remove this ASAP
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
    use std::thread::sleep;

    use chrono::{Duration, Utc};
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

    #[test]
    fn test_datum_generator_new_and_generate() {
        // Create a generator closure
        let mut counter = 0;
        let generator = move |_: DateTime<Utc>| -> DatumValue {
            counter += 1;
            DatumValue::Int(counter)
        };

        let datum_generator = DatumGenerator::new(Box::new(generator), DatumUnit::Unitless);

        // Generate a datum and verify its contents
        let datum1 = datum_generator.generate();
        assert_eq!(datum1.value, DatumValue::Int(1));
        assert_eq!(datum1.unit, DatumUnit::Unitless);

        // Generate another datum and verify the counter incremented
        let datum2 = datum_generator.generate();
        assert_eq!(datum2.value, DatumValue::Int(2));
        assert_eq!(datum2.unit, DatumUnit::Unitless);
    }

    #[test]
    fn test_datum_generator() {
        // Create a DatumGenerator with a mutable closure
        let generator = Box::new(move |now: DateTime<Utc>| -> DatumValue {
            DatumValue::Int(now.timestamp() as i32)
        });
        let datum_generator = DatumGenerator::new(generator, DatumUnit::Unitless);

        // Generate a Datum and test its properties
        let datum = datum_generator.generate();
        match datum.value {
            DatumValue::Int(value) => assert!(value >= 0), // assuming current timestamp is positive
            _ => panic!("Unexpected DatumValue type"),
        }
        assert_eq!(datum.unit, DatumUnit::Unitless);
    }

    #[test]
    fn test_generator_linearly_increasing_f32() {
        let env = Environment::default();
        let slope = 2.0_f32; // Increased slope
        let noise = 0.01_f32; // Reduced noise
        let unit = DatumUnit::DegreesC;

        let generator = env.generator_linearly_increasing_f32(slope, noise, unit);

        // Generate a datum, then wait and generate another
        let datum1 = generator.generate();

        // Increase the delay
        sleep(Duration::seconds(1).to_std().unwrap());
        let datum2 = generator.generate();

        // Check the unit
        assert_eq!(datum1.unit, unit);
        assert_eq!(datum2.unit, unit);

        // Check the value type and linear increase
        if let DatumValue::Float(value1) = datum1.value {
            if let DatumValue::Float(value2) = datum2.value {
                assert!(
                    value2 > value1,
                    "Value did not increase: value2 = {}, value1 = {}",
                    value2,
                    value1
                );
            } else {
                panic!("Datum2 value is not a float");
            }
        } else {
            panic!("Datum1 value is not a float");
        }
    }

    #[test]
    fn test_generator_linearly_decreasing_f32() {
        let env = Environment::default();
        let slope = 2.0_f32; // Slope for the linear decrease
        let noise = 0.01_f32; // Noise factor
        let unit = DatumUnit::DegreesC; // Unit for the Datum

        let generator = env.generator_linearly_decreasing_f32(slope, noise, unit);

        // Generate a datum, then wait and generate another
        let datum1 = generator.generate();

        // Introduce a delay
        sleep(Duration::seconds(1).to_std().unwrap());
        let datum2 = generator.generate();

        // Verify the unit
        assert_eq!(datum1.unit, unit);
        assert_eq!(datum2.unit, unit);

        // Verify the type and decreasing values
        if let DatumValue::Float(value1) = datum1.value {
            if let DatumValue::Float(value2) = datum2.value {
                assert!(
                    value2 < value1,
                    "Value did not decrease: value2 = {}, value1 = {}",
                    value2,
                    value1
                );
            } else {
                panic!("Datum2 value is not a float");
            }
        } else {
            panic!("Datum1 value is not a float");
        }
    }

    #[test]
    fn test_get_with_existing_generator() {
        let mut env = Environment::default();
        let id = Id::new("test_id");
        let unit = DatumUnit::DegreesC;

        // Create a generator for this Id
        let f = |_| -> DatumValue { DatumValue::Int(42) };
        let generator = DatumGenerator::new(Box::new(f), unit);
        env.attributes.lock().unwrap().insert(id.clone(), generator);

        // Test get method with existing generator
        let datum = env.get(&id, DatumValueType::Int, unit);
        assert_eq!(datum.value, DatumValue::Int(42));
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_get_with_new_bool_generator() {
        let mut env = Environment::default();
        let id = Id::new("new_bool_id");
        let unit = DatumUnit::Unitless;

        // Test get method with new generator for bool type
        let datum = env.get(&id, DatumValueType::Bool, unit);
        match datum.value {
            DatumValue::Bool(_) => (),
            _ => panic!("Expected Bool, found {:?}", datum.value),
        }
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_get_with_new_int_generator() {
        let mut env = Environment::default();
        let id = Id::new("new_int_id");
        let unit = DatumUnit::PoweredOn;

        // Test get method with new generator for Int type
        let datum = env.get(&id, DatumValueType::Int, unit);
        match datum.value {
            DatumValue::Int(_) => (),
            _ => panic!("Expected Int, found {:?}", datum.value),
        }
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_get_with_new_float_generator() {
        let mut env = Environment::default();
        let id = Id::new("new_float_id");
        let unit = DatumUnit::DegreesC;

        // Test get method with new generator for float type
        let datum = env.get(&id, DatumValueType::Float, unit);
        match datum.value {
            DatumValue::Float(_) => (),
            _ => panic!("Expected Float, found {:?}", datum.value),
        }
        assert_eq!(datum.unit, unit);
    }
}
