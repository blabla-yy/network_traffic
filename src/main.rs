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
// LOOPBACK 回环
// PHYSICAL_ONLY 物理
fn main() {
    let mut traffic = NetworkTraffic::new();

    traffic.start_to_collect();

    let mut count = 0;
    loop {
        let frames = traffic.take();

        let mut total_download = 0;
        let mut total_upload = 0;
        for frame in frames {
            // println!("pid: {}, download: {} KB/s, upload: {} KB/s", frame.pid, frame.download_length / 1024, frame.upload_length / 1024);
            total_upload = total_upload + frame.upload_length;
            total_download = total_download + frame.download_length;
        }
        println!("download: {} upload: {}", format_speed(total_download), format_speed(total_upload));
        std::thread::sleep(Duration::from_secs(1));
        count = count + 1;
        if count > 5 {
            traffic.stop();
            break;
        }
    }
    println!("stopped");
}

fn format_speed(size: usize) -> String {
    if size > 1024 * 1024 {
        format!("{} {}/s", size / (1024 * 1024), "MB")
    } else if size > 1024 {
        format!("{} {}/s", size / 1024, "KB")
    } else {
        format!("{} {}/s", size, "B")
    }
}