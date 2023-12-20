use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use device::Device;

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    /// The `act` command tells the actuator to perform some action.
    ///
    /// The action can be anything (turning on a light, setting a thermostat target temperature to
    /// some value, locking a door, etc.), so the `command` is a `String` which can be formatted in
    /// any way by the `Controller` and parsed in any way by the `Actuator`.
    ///
    /// In the "real world", this would perform some actual, physical action.
    ///
    /// In our example MVP, this sends a command to the `Environment` which mutates its state.
    fn act(sensor: device::Id, command: String);

    /// Responds to all incoming requests by forwarding them to the `Environment`.
    fn respond(&self, listener: TcpListener) {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut request = String::new();

            BufReader::new(&mut stream).read_line(&mut request).unwrap();
            println!("{} received request: {}", self.get_name(), request.trim());

            let ack = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(ack.as_bytes()).unwrap();
        }
    }
}
