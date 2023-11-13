use std::net::IpAddr;

use pnet::datalink::NetworkInterface;
use pnet::ipnetwork::IpNetwork;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use crate::traffic::config::ProtocolType;
use crate::traffic::sys_info::ProtocolPort;

#[derive(Debug, Clone)]
pub struct Frame {
    pub interface_name: String,
    pub data_length: usize,
    pub is_upload: bool,

    pub protocol: ProtocolType,
    pub source_ip: IpAddr,
    pub source_port: u16,
    pub destination_ip: IpAddr,
    pub destination_port: u16,
}

impl Frame {
    pub fn local_port(&self) -> u16 {
        let mut port = self.source_port;
        if !self.is_upload {
            port = self.destination_port;
        }
        port
    }

    pub fn protocol_port(&self) -> ProtocolPort {
        ProtocolPort::new(self.protocol, self.local_port())
    }
}

// 获取端口
fn get_port(protocol: IpNextHeaderProtocol, packet: &[u8]) -> Option<(u16, u16)> {
    match protocol {
        IpNextHeaderProtocols::Udp => {
            let udp = UdpPacket::new(packet).unwrap();
            Some((udp.get_source(), udp.get_destination()))
        }
        IpNextHeaderProtocols::Tcp => {
            let tcp = TcpPacket::new(packet).unwrap();
            Some((tcp.get_source(), tcp.get_destination()))
        }
        _ => return None,
    }
}

fn is_upload(network_interface_ips: &[IpNetwork], source: IpAddr) -> bool {
    return network_interface_ips
        .iter()
        .any(|ip_network| ip_network.ip() == source);
}

fn is_upload_by_macaddr(interface: &NetworkInterface, ethernet: &EthernetPacket) -> bool {
    interface.mac
        .map(|addr| addr == ethernet.get_source())
        .unwrap_or_else(|| {
            println!("unknown interface mac");
            return false
        })
}

pub fn handle_ethernet_frame(interface: &NetworkInterface, ethernet: &EthernetPacket) -> Option<Frame> {
    return match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            let header = Ipv4Packet::new(ethernet.payload()).unwrap();
            let next_protocol = header.get_next_level_protocol();
            let (source_port, destination_port) = get_port(next_protocol, header.payload())?;
            let source_ip = IpAddr::V4(header.get_source());
            let is_upload = is_upload(&interface.ips, source_ip);
            let by_mac = is_upload_by_macaddr(interface, ethernet);
            if is_upload != by_mac {
                println!("not equals")
            }
            let protocol = match next_protocol {
                IpNextHeaderProtocols::Udp => {
                    ProtocolType::Udp
                }
                IpNextHeaderProtocols::Tcp => {
                    ProtocolType::Tcp
                }
                _ => {
                    println!("neither tcp nor udp");
                    return None;
                }
            };
            Some(Frame {
                interface_name: interface.name.to_string(),
                data_length: ethernet.packet().len(),
                is_upload: is_upload,
                protocol: protocol,
                source_ip,
                source_port,
                destination_ip: IpAddr::V4(header.get_destination()),
                destination_port,
            })
        }
        EtherTypes::Ipv6 => {
            let header = Ipv6Packet::new(ethernet.payload()).unwrap();
            let next_header = header.get_next_header();
            let (source_port, destination_port) = get_port(next_header, header.payload())?;
            let source_ip = IpAddr::V6(header.get_source());

            let is_upload = is_upload(&interface.ips, source_ip);
            let by_mac = is_upload_by_macaddr(interface, ethernet);
            if is_upload != by_mac {
                println!("not equals")
            }

            let protocol = match next_header {
                IpNextHeaderProtocols::Udp => {
                    ProtocolType::Udp
                }
                IpNextHeaderProtocols::Tcp => {
                    ProtocolType::Tcp
                }
                _ => {
                    println!("neither tcp nor udp");
                    return None;
                }
            };
            Some(Frame {
                interface_name: interface.name.to_string(),
                data_length: ethernet.packet().len(),
                is_upload: is_upload,
                protocol: protocol,
                source_ip,
                source_port,
                destination_ip: IpAddr::V6(header.get_destination()),
                destination_port,
            })
        }
        _ => {
            None
        }
    };
}