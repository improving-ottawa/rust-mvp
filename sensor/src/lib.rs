use std::io::BufReader;
use std::io::{BufRead, Write};
use std::net::{IpAddr, TcpListener};

use mdns_sd::ServiceInfo;

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
    fn get_name(&self) -> &Name;

    /// Returns the unique ID of this `Sensor`.
    fn get_id(&self) -> &Id;

    /// Registers this `Sensor` with mDNS in the `_sensor` group.
    fn register(&self, ip: IpAddr, port: u16) {
        let mdns = mdns_sd::ServiceDaemon::new().unwrap();
        let host = ip.clone().to_string();
        let name = &self.get_name().0;
        let domain = "_sensor._tcp.local.";

        println!("\nRegistering new sensor via mDNS at {}.{}", name, domain);

        let my_service =
            ServiceInfo::new(domain, name.as_str(), host.as_str(), ip, port, None).unwrap();

        mdns.register(my_service).unwrap()
    }

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    fn listener(&self, ip: IpAddr, port: u16) -> TcpListener {
        let host = ip.clone().to_string();
        let address = format!("{}:{}", host, port);
        let name = &self.get_name().0;

        println!("\nCreating new sensor {} at {}", name, address);
        println!("Ask this sensor for data with: curl {}", address);

        TcpListener::bind(address).unwrap()
    }

    /// Registers this `Sensor` with mDNS and binds it to the specified `ip` and `port.
    fn bind(&self, ip: IpAddr, port: u16) -> TcpListener {
        self.register(ip, port);
        self.listener(ip, port)
    }

    /// Responds to all incoming requests with the latest `Datum`.
    fn respond(&self, listener: TcpListener) {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut request = String::new();

            BufReader::new(&mut stream).read_line(&mut request).unwrap();
            println!("received request: {}", request.trim());

            let contents = self.get_datum().to_string();
            let ack = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
                contents.len(),
                contents
            );
            stream.write_all(ack.as_bytes()).unwrap();
        }
    }
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

    struct Thermometer {
        id: Id,
        name: Name,
    }

    impl Thermometer {
        fn new() -> Thermometer {
            Thermometer {
                id: Id::new("should be random"),
                name: Name::new("Thermometer"),
            }
        }
    }

    impl Sensor for Thermometer {
        fn get_datum(&self) -> Datum {
            // in our example, this should query the Environment
            // in this test, we just return a constant value
            Datum::new_now(DatumValue::Float(42.0), Some(DatumUnit::DegreesC))
        }

        fn get_name(&self) -> &Name {
            &self.name
        }

        fn get_id(&self) -> &Id {
            &self.id
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
        assert_eq!(thermometer.get_name(), &Name::new("Thermometer"))
    }

    #[test]
    fn test_get_id() {
        let thermometer = Thermometer::new();
        assert_eq!(thermometer.get_id(), &Id::new("should be random"))
    }
}
