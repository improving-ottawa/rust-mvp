use std::sync::Mutex;

use chrono::{DateTime, Utc};

use datum::{Datum, DatumUnit, DatumValue};

pub struct DatumGenerator {
    #[allow(dead_code)] // remove this ASAP
    generator: Mutex<Box<dyn FnMut(DateTime<Utc>) -> DatumValue>>,
    #[allow(dead_code)] // remove this ASAP
    unit: DatumUnit,
}

impl DatumGenerator {
    #[allow(dead_code)] // remove this ASAP
    pub(crate) fn new(
        generator: Box<dyn FnMut(DateTime<Utc>) -> DatumValue>,
        unit: DatumUnit,
    ) -> DatumGenerator {
        DatumGenerator {
            generator: Mutex::new(generator),
            unit,
        }
    }

    #[allow(dead_code)] // remove this ASAP
    pub(crate) fn generate(&self) -> Datum {
        let now = Utc::now();
        let mut generator = self.generator.lock().unwrap();
        let value = (*generator)(now);
        Datum::new(value, self.unit, now)
    }
}

pub mod time_dependent {
    use chrono::{DateTime, Utc};
    use rand::{thread_rng, Rng};

    use datum::{DatumUnit, DatumValue};

    use crate::generator::DatumGenerator;

    #[allow(dead_code)] // remove this ASAP
    pub fn f32_linear_increasing(slope: f32, noise: f32, unit: DatumUnit) -> DatumGenerator {
        let start = Utc::now().timestamp_millis();
        let mut rng = thread_rng();

        let f = move |now: DateTime<Utc>| -> DatumValue {
            let delta = now.timestamp_millis() - start;
            let noise_factor = rng.gen_range(-1.0..1.0) * noise;
            DatumValue::Float(delta as f32 * slope + noise_factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }

    #[allow(dead_code)] // remove this ASAP
    pub fn f32_linear_decreasing(slope: f32, noise: f32, unit: DatumUnit) -> DatumGenerator {
        let start = Utc::now().timestamp_millis();
        let mut rng = thread_rng();

        let f = move |now: DateTime<Utc>| -> DatumValue {
            let delta = now.timestamp_millis() - start;
            let noise_factor = rng.gen_range(-1.0..1.0);
            DatumValue::Float(-1.0 * (delta as f32) * slope + noise * noise_factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }

    pub fn i32_random_walk(slope: i32, unit: DatumUnit) -> DatumGenerator {
        let start = Utc::now().timestamp_millis();
        let mut rng = thread_rng();

        let f = move |now: DateTime<Utc>| -> DatumValue {
            let delta = ((now.timestamp_millis() - start) / 1000) as i32;
            let factor = if rng.gen_bool(0.5) { 1 } else { -1 };
            DatumValue::Int(delta * slope * factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }
}

pub fn bool_alternating(unit: DatumUnit) -> DatumGenerator {
    let mut latest_value = false;

    let f = move |_| -> DatumValue {
        latest_value = !latest_value;
        DatumValue::Bool(latest_value)
    };

    DatumGenerator::new(Box::new(f), unit)
}

#[cfg(test)]
mod generator_tests {
    use std::thread::sleep;

    use chrono::{Duration, Utc};

    use datum::DatumUnit;

    use super::*;

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
        let slope = 2.0_f32; // Increased slope
        let noise = 0.01_f32; // Reduced noise
        let unit = DatumUnit::DegreesC;

        let generator = time_dependent::f32_linear_increasing(slope, noise, unit);

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
        let slope = 2.0_f32; // Slope for the linear decrease
        let noise = 0.01_f32; // Noise factor
        let unit = DatumUnit::DegreesC; // Unit for the Datum

        let generator = time_dependent::f32_linear_decreasing(slope, noise, unit);

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
}
