use std::collections::HashMap;

/// A test-only example environment which produces data detected by `Sensor`s.
///
/// The `Environment` can be mutated by `Actuator`s.
struct Environment {
    bools: HashMap<String, bool>
}

impl Environment {

    fn new() -> Environment {
        Environment { bools: HashMap::new() }
    }

    fn set_bool(&mut self, name: &str, value: bool) {
        self.bools.insert(String::from(name), value);
    }

    fn get_bool(&self, name: &str) -> Option<bool> {
        self.bools.get(name).map(|&v| v)
    }

    fn toggle_bool(&mut self, name: &str) {
        match self.get_bool(name) {
            Some(current) => self.set_bool(name, !current),
            None => ()
        }
    }

    // TODO add similar getters and setters for int, float, string values

    // TODO add random data generation as necessary

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get_bool() {
        let mut env = Environment::new();

        env.set_bool("should be true", true);
        env.set_bool("should be false", false);

        assert_eq!(env.get_bool("should be true"), Some(true));
        assert_eq!(env.get_bool("should be false"), Some(false));
        assert_eq!(env.get_bool("should not exist"), None);
    }

    #[test]
    fn test_toggle_bool() {
        let mut env = Environment::new();

        env.set_bool("should be false", true);
        env.toggle_bool("should be false");
        assert_eq!(env.get_bool("should be false"), Some(false));

        env.set_bool("should be true", false);
        env.toggle_bool("should be true");
        assert_eq!(env.get_bool("should be true"), Some(true));

        env.toggle_bool("should not exist");
        assert_eq!(env.get_bool("should not exist"), None);
    }
}
