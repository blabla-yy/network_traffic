extern crate pnet;


use std::time::Duration;

use nix::unistd::Uid;

use crate::traffic::network_traffic::{NetworkTraffic, ProcessPacketLength, ProcessStatistics};

pub mod traffic;


#[no_mangle]
pub extern fn take(f: extern fn(ProcessStatistics)) {
    let mut traffic = NetworkTraffic::new();

    println!("is root user: {}", Uid::effective().is_root());

    traffic.start_to_collect();

    loop {
        match traffic.take() {
            None => {
                println!("none");
                f(ProcessStatistics {
                    length: 0,
                    list: vec![].as_ptr(),
                    elapse_millisecond: 0,
                })
            }
            Some(item) => {
                f(item)
            }
        };
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