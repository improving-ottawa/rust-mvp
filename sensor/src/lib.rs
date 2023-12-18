use datum::Datum;

/// A Sensor collects data from the Environment.
trait Sensor {

    /// To get data out of a sensor, we call `sensor.get_datum()`.
    ///
    /// In the "real world", this would poll some actual physical sensor for a data point.
    ///
    /// In our example MVP, this queries the `Environment` for data.
    fn get_datum(&self) -> Datum;

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
    }

    #[test]
    fn test_get_datum() {
        let thermometer = Thermometer::new();
        let datum = thermometer.get_datum();

        assert_eq!(datum.value, DatumValue::Float(42.0));
        assert_eq!(datum.unit, Some(DatumUnit::DegreesC));
    }
}