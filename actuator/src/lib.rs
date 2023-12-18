/// An Actuator mutates the Environment.
trait Actuator {
    /// The `act` command tells the actuator to perform some action.
    ///
    /// The action can be anything (turning on a light, setting a thermostat target temperature to
    /// some value, locking a door, etc.), so the `command` is a `String` which can be formatted in
    /// any way by the `Controller` and parsed in any way by the `Actuator`.
    ///
    /// In the "real world", this would perform some actual, physical action.
    ///
    /// In our example MVP, this sends a command to the `Environment` which mutates its state.
    fn act(command: String);
}
