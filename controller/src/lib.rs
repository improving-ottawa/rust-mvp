use std::collections::HashSet;

use datum::Datum;

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so historic data
struct Controller {

    /// Holds data queried from `Sensor`s
    data: HashSet<SensorHistory>

}

struct SensorHistory {
    name: String,
    data: Vec<Datum>
}

// TODO add getters here as needed once we need to query the Controller's state for the HTML page