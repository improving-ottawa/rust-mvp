use std::collections::HashMap;

use datum::Datum;

/// A test-only example environment which produces data detected by `Sensor`s.
///
/// The `Environment` can be mutated by `Actuator`s.
#[derive(Default)] // gives us an "empty" Environment with Environment::default()
struct Environment {
    attributes: HashMap<sensor::Id, Datum>,
}

#[allow(dead_code)] // remove ASAP
impl Environment {
    fn new() -> Environment {
        Environment::default()
    }

    fn set(&mut self, id: &str, value: Datum) {
        self.attributes.insert(sensor::Id::new(id), value);
    }

    fn get(&self, id: &str) -> Option<&Datum> {
        self.attributes.get(&sensor::Id::new(id))
    }

    // TODO add random data generation as necessary
}

#[cfg(test)]
mod env_tests {
    use datum::DatumValue;

    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut env = Environment::new();

        let datum = Datum::new_now(DatumValue::Int(42), None);

        env.set("exists", datum);

        let got = env.get("exists");

        assert!(got.is_some());

        let got = got.unwrap();
        let value = &got.value;
        let unit = &got.unit;

        assert_eq!(value, &DatumValue::Int(42));
        assert_eq!(unit, &None);
    }
}
