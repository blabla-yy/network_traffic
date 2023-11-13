use pnet::datalink::NetworkInterface;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ProtocolType {
    Tcp,
    Udp,
}

#[warn(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum InterfaceType {
    Lo,
    // 不支持
    En,
    Utun,
    Awdl,
    Llw,
    Bridge,
    P2p,
}

impl InterfaceType {
    pub fn filter(&self, network_interface: &NetworkInterface) -> bool {
        match self {
            InterfaceType::Lo => {
                network_interface.name.starts_with("lo") || network_interface.is_loopback()
            }
            InterfaceType::En => {
                network_interface.name.starts_with("en")
            }
            InterfaceType::Utun => {
                network_interface.name.starts_with("utun")
            }
            InterfaceType::Awdl => {
                network_interface.name.starts_with("awdl")
            }
            InterfaceType::Llw => {
                network_interface.name.starts_with("llw")
            }
            InterfaceType::Bridge => {
                network_interface.name.starts_with("bridge")
            }
            InterfaceType::P2p => {
                network_interface.name.starts_with("p2p") || network_interface.is_point_to_point()
            }
        }
    }
}
