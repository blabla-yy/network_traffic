extern crate pnet;


use std::time;
use std::time::Duration;

use crate::traffic::network_traffic::{NetworkTraffic, ProcessPacketLength, ProcessStatistics};

pub mod traffic;


#[no_mangle]
pub extern fn take(f: extern fn(ProcessStatistics)) {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    loop {
        let start = time::Instant::now();
        let mut list = traffic.take();
        let elapse = start.elapsed().as_secs();
        if list.is_empty() {
            f(ProcessStatistics {
                length: 0,
                list: vec![].as_ptr(),
                elapse_millisecond: 0,
            })
        } else {
            let item = ProcessStatistics {
                length: list.len(),
                list: list.as_ptr(),
                elapse_millisecond: elapse,
            };
            std::mem::forget(list);
            f(item)
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[no_mangle]
pub extern "C" fn free_array(statistics: ProcessStatistics) {
    drop(unsafe {
        let v = Vec::from_raw_parts(statistics.list as *mut ProcessPacketLength, statistics.length, statistics.length);
        println!("drop {}, len {}", v.len(), statistics.length);
    });
}