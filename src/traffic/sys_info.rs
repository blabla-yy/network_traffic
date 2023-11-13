use std::collections::HashMap;

use netstat2::{AddressFamilyFlags, get_sockets_info, ProtocolFlags, ProtocolSocketInfo};


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

pub fn  get_port_process_map(frames: &Vec<crate::traffic::analyze::Frame>) -> HashMap<u16, u32> {
    let mut map: HashMap<u16, u32> = HashMap::new();
    for item in frames {
        map.insert(item.local_port(), 0);
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
                        if map.contains_key(&tcp_si.local_port) {
                            map.insert(tcp_si.local_port, socket.associated_pids[0]);
                        }
                    }
                    ProtocolSocketInfo::Udp(udp_si) => {
                        if map.contains_key(&udp_si.local_port) {
                            map.insert(udp_si.local_port, socket.associated_pids[0]);
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