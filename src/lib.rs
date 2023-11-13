extern crate pnet;


use std::time::Duration;

use crate::traffic::network_traffic::{NetworkTraffic, ProcessPacketLength, ProcessStatistics};

pub mod traffic;


#[no_mangle]
pub extern fn take(f: extern fn(ProcessStatistics)) {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    // 这里循环主要是为了简化与C交互，否则需要处理NetworkTraffic实例。
    loop {
        let statistics = traffic.take();
        f(statistics);
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[no_mangle]
pub extern "C" fn free_data(statistics: ProcessStatistics) {
    statistics.free();
}