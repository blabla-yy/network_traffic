extern crate pnet;


use std::time::Duration;

use crate::network::network_traffic::NetworkTraffic;

mod network;

// UP 在线设备
// VIRTUAL 虚拟
// LOOPBACK 回环
// PHYSICAL_ONLY 物理
fn main() {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    let mut count = 0;
    loop {
        let statistics = traffic.take();
        match serde_json::to_string(&statistics) {
            Ok(json) => {
                println!("{}", json);
            }
            Err(err) => {
                eprintln!("serialize json error {}", err);
            }
        }
        std::thread::sleep(Duration::from_secs(1));
        count = count + 1;
    }
}
