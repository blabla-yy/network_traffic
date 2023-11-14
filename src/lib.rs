extern crate pnet;


use std::mem::forget;
use std::time::Duration;

use crate::network::network_traffic::{NetworkTraffic, ProcessPacketLength};

pub mod network;

#[derive(Debug)]
#[repr(C)]
pub struct ProcessStatisticsFFI {
    pub length: usize,
    pub list: *const ProcessPacketLength,
    pub total_upload: u64,
    pub total_download: u64,
}

#[no_mangle]
pub extern fn take(f: extern fn(ProcessStatisticsFFI)) {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    loop {
        let statistics = traffic.take();
        let ptr = statistics.list.as_ptr();
        let len = statistics.list.len();

        forget(statistics.list);

        f(ProcessStatisticsFFI {
            length: len,
            list: ptr,
            total_upload: statistics.total_upload,
            total_download: statistics.total_download,
        });
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[no_mangle]
pub extern "C" fn free_data(statistics: ProcessStatisticsFFI) {
    drop(unsafe {
        let v = Vec::from_raw_parts(statistics.list as *mut ProcessPacketLength, statistics.length, statistics.length);
    });
}