/// A Sensor collects data from the Environment.
trait Sensor<T> {

    /// To get data out of a sensor, we call `sensor.value()`.
    ///
    /// In the "real world", this would poll some actual physical device for data.
    ///
    /// In our example MVP, this queries the `Environment` for data.
    fn get_value(&self) -> T;

}

#[cfg(test)]
mod tests {
    use super::*;

    struct Thermometer {}

    impl Thermometer {
        fn new() -> Thermometer {
            Thermometer {}
        }
    }

    impl Sensor<f32> for Thermometer {
        fn get_value(&self) -> f32 {
            // in our example, this should query the Environment
            // in this test, we just return a constant value
            42.0
        }
    }

    #[test]
    fn test_get_value() {
        let thermometer = Thermometer::new();
        assert_eq!(thermometer.get_value(), 42.0);
    }
}