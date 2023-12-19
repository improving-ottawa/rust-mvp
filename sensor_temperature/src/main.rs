use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use uuid::Uuid;

use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

fn main() {
    // Create a daemon
    let mdns = mdns_sd::ServiceDaemon::new().expect("Failed to create daemon");

    let ip = local_ip_address::local_ip().unwrap();
    let host = ip.clone().to_string();

    let port = 8787;

    let address = format!("{}:{}", host, port);

    let sensor_id = Uuid::new_v4().to_string();
    let sensor_name = "temperature_sensor";

    let endpoint = "/datum";

    println!("\nCreating new sensor {} at {}", sensor_name, address);
    println!(
        "Ask this sensor for data with: curl {}{}",
        address, endpoint
    );

    // Create a service info.
    let service_type = "_sensor._tcp.local.";

    let my_service =
        mdns_sd::ServiceInfo::new(service_type, sensor_name, host.as_str(), ip, port, None)
            .unwrap();

    // Register with the daemon, which publishes the service.
    mdns.register(my_service)
        .expect("Failed to register our service");

    let listener = TcpListener::bind(address).unwrap();

    let sensor = TemperatureSensor::new(sensor_id.as_str(), sensor_name);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut request = String::new();

        BufReader::new(&mut stream).read_line(&mut request).unwrap();
        println!("received request: {}", request.trim());

        let contents = sensor.get_datum().to_string();
        let ack = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            contents.len(),
            contents
        );
        stream.write_all(ack.as_bytes()).unwrap();
    }
}
