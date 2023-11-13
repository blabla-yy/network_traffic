extern crate pnet;


use std::time::Duration;

use crate::traffic::network_traffic::{NetworkTraffic, ProcessPacketLength, ProcessStatistics};

pub mod traffic;


#[no_mangle]
pub extern fn take(f: extern fn(ProcessStatistics)) {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    loop {
        let statistics = traffic.take();
        f(statistics);
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[no_mangle]
pub extern "C" fn free_array(statistics: ProcessStatistics) {
    statistics.free();
}