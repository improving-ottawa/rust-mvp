use datum::Datum;

/// A Sensor collects data from the Environment.
pub trait Sensor {
    /// To get data out of a sensor, we call `sensor.get_datum()`.
    ///
    /// In the "real world", this would poll some actual physical sensor for a data point.
    ///
    /// In our example MVP, this queries the `Environment` for data.
    fn get_datum(&self) -> Datum;

    /// Returns the user-friendly name of this `Sensor`.
    fn get_name(&self) -> Name;

    /// Returns the unique ID of this `Sensor`.
    fn get_id(&self) -> Id;
}

#[derive(PartialEq, Debug)]
pub struct Name(String);

impl Name {
    #[allow(dead_code)] // remove ASAP
    pub fn new(name: &str) -> Name {
        Name(String::from(name))
    }
}

#[derive(PartialEq, Debug, Eq, Hash)]
pub struct Id(String);

impl Id {
    #[allow(dead_code)] // remove ASAP
    pub fn new(id: &str) -> Id {
        Id(String::from(id))
    }
}

#[cfg(test)]
mod sensor_tests {
    use datum::{DatumUnit, DatumValue};

    use super::*;

    struct Thermometer {}

    impl Thermometer {
        fn new() -> Thermometer {
            Thermometer {}
        }
    }

    impl Sensor for Thermometer {
        fn get_datum(&self) -> Datum {
            // in our example, this should query the Environment
            // in this test, we just return a constant value
            Datum::new_now(DatumValue::Float(42.0), Some(DatumUnit::DegreesC))
        }

        fn get_name(&self) -> Name {
            Name::new("Thermometer")
        }

        fn get_id(&self) -> Id {
            Id::new("should be random")
        }
    }

    #[test]
    fn test_get_datum() {
        let thermometer = Thermometer::new();
        let datum = thermometer.get_datum();

        assert_eq!(datum.value, DatumValue::Float(42.0));
        assert_eq!(datum.unit, Some(DatumUnit::DegreesC));
    }

    #[test]
    fn test_get_name() {
        let thermometer = Thermometer::new();
        assert_eq!(thermometer.get_name(), Name::new("Thermometer"))
    }

    #[test]
    fn test_get_id() {
        let thermometer = Thermometer::new();
        assert_eq!(thermometer.get_id(), Id::new("should be random"))
    }
}
