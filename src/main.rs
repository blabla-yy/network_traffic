extern crate pnet;



use std::time::Duration;

use crate::traffic::network_traffic::NetworkTraffic;

mod traffic;


#[derive(Debug, Clone)]
#[repr(C)]
pub struct ProcessPacketLength {
    // 递归至非1的父进程ID
    pub pid: u32,
    pub upload_length: usize,
    pub download_length: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct ProcessStatistics {
    pub length: usize,
    // list: Vec<ProcessPacketLength>,
    pub list: *const ProcessPacketLength,
    // 本次收集的数据，使用的时间
    pub elapse_millisecond: u128,
}

// 所有网卡信息
// UP 在线设备
// VIRTUAL 虚拟
// LOOPBACK
// PHYSICAL_ONLY 物理
fn main() {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    loop {
        let frames = traffic.take();
        println!("{:?}", frames);
        std::thread::sleep(Duration::from_secs(1));
    }
}