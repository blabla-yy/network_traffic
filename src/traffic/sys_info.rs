use std::collections::HashMap;

use netstat2::{AddressFamilyFlags, get_sockets_info, ProtocolFlags, ProtocolSocketInfo};

use crate::traffic::config::ProtocolType;

fn get_process_by_port(port: u16) -> Option<Vec<u32>> {
    let sockets_info = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP).unwrap();

    for socket in sockets_info {
        match socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                if tcp_si.local_port == port {
                    return Some(socket.associated_pids);
                }
            }
            ProtocolSocketInfo::Udp(udp_si) => {
                if udp_si.local_port == port {
                    return Some(socket.associated_pids);
                }
            }
        }
    }

    None
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct ProtocolPort {
    pub protocol: ProtocolType,
    pub port: u16,
}

impl ProtocolPort {
    pub fn new(protocol: ProtocolType, port: u16) -> Self {
        ProtocolPort {
            protocol,
            port,
        }
    }
}

pub fn get_port_process_map(frames: &Vec<crate::traffic::analyze::Frame>) -> HashMap<ProtocolPort, u32> {
    let mut map: HashMap<ProtocolPort, u32> = HashMap::new();
    for item in frames {
        map.insert(ProtocolPort::new(item.protocol, item.local_port()), 0);
    }

    let sockets_info = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP | ProtocolFlags::UDP);
    match sockets_info {
        Ok(sockets_info) => {
            for socket in sockets_info {
                if socket.associated_pids.is_empty() {
                    continue;
                }
                match socket.protocol_socket_info {
                    ProtocolSocketInfo::Tcp(tcp_si) => {

                        let protocol_port = ProtocolPort::new(ProtocolType::Tcp, tcp_si.local_port);
                        // println!("exists {:?}", protocol_port);
                        if map.contains_key(&protocol_port) {
                            map.insert(protocol_port, socket.associated_pids[0]);
                        }
                    }
                    ProtocolSocketInfo::Udp(udp_si) => {
                        let protocol_port = ProtocolPort::new(ProtocolType::Udp, udp_si.local_port);
                        // println!("exists {:?}", protocol_port);
                        if map.contains_key(&protocol_port) {
                            map.insert(protocol_port, socket.associated_pids[0]);
                        }
                    }
                }
            }
        }
        Err(err) => {
            println!("get sockets info error: {:?}", err)
        }
    }

    map
}