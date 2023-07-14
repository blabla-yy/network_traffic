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
    pub list: *const ProcessPacketLength,
    // 本次收集的数据，使用的时间
    pub elapse_millisecond: u128,
}

// UP 在线设备
// VIRTUAL 虚拟
// LOOPBACK
// PHYSICAL_ONLY 物理
fn main() {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    loop {
        let frames = traffic.take();

        let mut total_download = 0;
        let mut total_upload = 0;
        for frame in frames {
            // println!("pid: {}, download: {} KB/s, upload: {} KB/s", frame.pid, frame.download_length / 1024, frame.upload_length / 1024);
            total_upload = total_upload + frame.upload_length;
            total_download = total_download + frame.download_length;
        }
        
        let download_display = if total_download > 1024 {
            format!("{} {}/s", total_download / 1024, "KB")
        } else {
            format!("{} {}/s", total_download, "B")
        };
        let upload_display = if total_upload > 1024 {
            format!("{} {}/s", total_upload / 1024, "KB")
        } else {
            format!("{} {}/s", total_upload, "B")
        };
        println!("download: {}/s upload: {}/s", download_display , upload_display);
        println!("-------");
        std::thread::sleep(Duration::from_secs(1));
    }
}