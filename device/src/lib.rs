use std::net::{IpAddr, TcpListener};

use mdns_sd::ServiceInfo;

/// A `Device` exists on the network and is discoverable via mDNS.
pub trait Device {
    /// Returns the user-friendly name of this `Device`.
    fn get_name(&self) -> &Name;

    /// Returns the unique ID of this `Device`.
    fn get_id(&self) -> &Id;

    /// Registers this `Device` with mDNS in the specified group.
    fn register(&self, ip: IpAddr, port: u16, group: &str) {
        let mdns = mdns_sd::ServiceDaemon::new().unwrap();
        let host = ip.clone().to_string();
        let name = &self.get_name().0;
        let domain = format!("{}._tcp.local.", group);

        println!("Registering new device via mDNS at {}.{}", name, domain);

        let my_service = ServiceInfo::new(
            domain.as_str(),
            name.as_str(),
            host.as_str(),
            ip,
            port,
            None,
        )
        .unwrap();

        mdns.register(my_service).unwrap()
    }

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    fn listener(&self, ip: IpAddr, port: u16) -> TcpListener {
        let host = ip.clone().to_string();
        let address = format!("{}:{}", host, port);
        let name = &self.get_name().0;

        println!("Creating new device '{}' at {}", name, address);

        TcpListener::bind(address).unwrap()
    }

    /// Registers this `Device` with mDNS in the specified `group` and binds it to listen at the specified `ip` and `port.
    fn bind(&self, ip: IpAddr, port: u16, group: &str) -> TcpListener {
        self.register(ip, port, group);
        self.listener(ip, port)
    }
}

#[derive(PartialEq, Debug)]
pub struct Name(pub String);

impl Name {
    #[allow(dead_code)] // remove ASAP
    pub fn new(name: &str) -> Name {
        Name(String::from(name))
    }
}

#[derive(PartialEq, Debug, Eq, Hash)]
pub struct Id(pub String);

impl Id {
    #[allow(dead_code)] // remove ASAP
    pub fn new(id: &str) -> Id {
        Id(String::from(id))
    }
}
