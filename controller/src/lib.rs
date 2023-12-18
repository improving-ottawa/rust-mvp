use std::collections::HashMap;

use datum::Datum;

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so historic data
#[allow(dead_code)] // remove ASAP
struct Controller {
    /// Holds data queried from `Sensor`s
    data: HashMap<sensor::Id, SensorHistory>,
}

#[allow(dead_code)] // remove ASAP
/// The `SensorHistory` holds the unique `id`, the user-friendly `name`, and some history of
/// `data` points.
struct SensorHistory {
    id: sensor::Id,
    name: sensor::Name,
    data: Vec<Datum>,
}

// TODO add getters here as needed once we need to query the Controller's state for the HTML page
