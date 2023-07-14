

use std::net::IpAddr;

use pnet::datalink::{NetworkInterface};

use pnet::ipnetwork::IpNetwork;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes, MutableEthernetPacket};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::util::MacAddr;

#[derive(Debug, Clone)]
pub struct Frame {
    pub interface_name: String,
    pub data_length: usize,
    pub is_upload: bool,
    pub source_ip: IpAddr,
    pub source_port: u16,
    pub destination_ip: IpAddr,
    pub destination_port: u16,
}

impl Frame {
    pub fn local_port(&self) -> u16{
        let mut port = self.source_port;
        if !self.is_upload {
            port = self.destination_port;
        }
        port
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
    {
        return network_interface_ips
            .iter()
            .any(|ip_network| ip_network.ip() == source);
    }
}

pub fn handle_ethernet_frame(interface: &NetworkInterface, ethernet: &EthernetPacket) -> Option<Frame> {
    return match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => {
            let header = Ipv4Packet::new(ethernet.payload()).unwrap();
            let (source_port, destination_port) =
                get_port(header.get_next_level_protocol(), header.payload())?;
            let source_ip = IpAddr::V4(header.get_source());
            Some(Frame {
                interface_name: interface.name.to_string(),
                data_length: ethernet.packet().len(),
                is_upload: is_upload(&interface.ips, source_ip),
                source_ip,
                source_port,
                destination_ip: IpAddr::V4(header.get_destination()),
                destination_port,
            })
        }
        EtherTypes::Ipv6 => {
            let header = Ipv6Packet::new(ethernet.payload()).unwrap();
            let (source_port, destination_port) =
                get_port(header.get_next_header(), header.payload())?;
            let source_ip = IpAddr::V6(header.get_source());
            Some(Frame {
                interface_name: interface.name.to_string(),
                data_length: ethernet.packet().len(),
                is_upload: is_upload(&interface.ips, source_ip),
                source_ip,
                source_port,
                destination_ip: IpAddr::V6(header.get_destination()),
                destination_port,
            })
        }
        _ => None,
    };
}

pub(crate) fn analyze_packet(interface: &NetworkInterface, packet: &[u8]) -> Option<Frame> {
    let mut buf: [u8; 1600] = [0u8; 1600];
    let mut fake_ethernet_frame = MutableEthernetPacket::new(&mut buf[..]).unwrap();

    let payload_offset;
    if cfg!(any(target_os = "macos", target_os = "ios"))
        && interface.is_up()
        && !interface.is_broadcast()
        && ((!interface.is_loopback() && interface.is_point_to_point())
        || interface.is_loopback())
    {
        if interface.is_loopback() {
            // The pnet code for BPF loopback adds a zero'd out Ethernet header
            payload_offset = 14;
        } else {
            // Maybe is TUN interface
            payload_offset = 0;
        }
        if packet.len() > payload_offset {
            let version = Ipv4Packet::new(&packet[payload_offset..])
                .unwrap()
                .get_version();
            if version == 4 {
                fake_ethernet_frame.set_destination(MacAddr(0, 0, 0, 0, 0, 0));
                fake_ethernet_frame.set_source(MacAddr(0, 0, 0, 0, 0, 0));
                fake_ethernet_frame.set_ethertype(EtherTypes::Ipv4);
                fake_ethernet_frame.set_payload(&packet[payload_offset..]);
                return handle_ethernet_frame(
                    &interface,
                    &fake_ethernet_frame.to_immutable(),
                );
            } else if version == 6 {
                fake_ethernet_frame.set_destination(MacAddr(0, 0, 0, 0, 0, 0));
                fake_ethernet_frame.set_source(MacAddr(0, 0, 0, 0, 0, 0));
                fake_ethernet_frame.set_ethertype(EtherTypes::Ipv6);
                fake_ethernet_frame.set_payload(&packet[payload_offset..]);
                return handle_ethernet_frame(
                    &interface,
                    &fake_ethernet_frame.to_immutable(),
                );
            }
        }
    }
    return
        handle_ethernet_frame(&interface, &EthernetPacket::new(packet).unwrap());
}
