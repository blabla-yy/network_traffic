use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::time::Instant;

use nix::unistd::Uid;
use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::NetworkInterface;

use crate::traffic::analyze::{analyze_packet, Frame};

pub struct NetworkTraffic {
    pub frames: Arc<Mutex<Vec<Frame>>>,
    pub another_frames: Arc<Mutex<Vec<Frame>>>,
    threads: Vec<JoinHandle<()>>,
    stop_signal: Arc<AtomicBool>,

    // only Ethernet interfaces
    only_en: bool,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ProcessPacketLength {
    pub pid: u32,
    pub upload_length: usize,
    pub download_length: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct ProcessStatistics {
    pub length: usize,
    pub list: *const ProcessPacketLength,
    pub elapse_millisecond: u64,
}

impl NetworkTraffic {
    pub fn new() -> Self {
        if !Uid::effective().is_root() {
            eprintln!("Not root user!");
        }
        NetworkTraffic {
            frames: Arc::new(Mutex::new(Vec::new())),
            another_frames: Arc::new(Mutex::new(Vec::new())),
            threads: Vec::new(),
            stop_signal: Arc::new(AtomicBool::new(false)),
            only_en: true,
        }
    }

    pub fn take(&mut self) -> Vec<ProcessPacketLength> {
// let mut tmp = self.another_frames.lock().ok()?;
        let mut tmp = Vec::new();
        {
            let mut frames = self.frames.lock().ok().unwrap();

            tmp.resize(frames.len(), Frame {
                interface_name: "".to_string(),
                data_length: 0,
                is_upload: false,
                source_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                source_port: 0,
                destination_ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                destination_port: 0,
            });
            tmp.clone_from_slice(frames.as_slice());
            frames.clear();
        }
        // 将tmp中的数据集合一下
        let mut map = HashMap::<u32, ProcessPacketLength>::new();
        let port_process = crate::traffic::sys_info::get_port_process_map(&tmp);
        for frame in tmp.iter() {
            match port_process.get(&frame.local_port()) {
                None => {}
                Some(pid) => {
                    let pid = *pid;
                    if pid == 0 {
                        // println!("pid是0 {:?}", frame);
                        continue;
                    }
                    let mut upload: usize = 0;
                    let mut download: usize = 0;
                    if frame.is_upload {
                        upload = frame.data_length;
                    } else {
                        download = frame.data_length;
                    }
                    let ex = map.entry(pid).or_insert(ProcessPacketLength {
                        pid,
                        upload_length: 0,
                        download_length: 0,
                    });
                    ex.upload_length = ex.upload_length + upload;
                    ex.download_length = ex.download_length + download;
                }
            }
        }

        let mut list = map.values().cloned().collect::<Vec<ProcessPacketLength>>();
        list.shrink_to_fit();
        tmp.clear();
        list
    }

    // 停止收集
    pub fn stop(&mut self) {
        println!("stop signal");
        self.stop_signal.swap(true, Ordering::Acquire);
    }

    // 将Frame推入Vec
    fn receive(signal: Arc<AtomicBool>, frames: Arc<Mutex<Vec<Frame>>>, rx: Receiver<Frame>) {
        loop {
            {
                let signal = signal.load(Ordering::Acquire);
                if signal {
                    println!("结束: {}", signal);
                    return;
                }
            }

            match rx.recv() {
                Ok(item) => {
                    match frames.lock().ok() {
                        None => {
                            println!("lock fail");
                        }
                        Some(mut v) => {
                            v.push(item);
                        }
                    }
                }
                Err(e) => {
                    println!("error while receiving {}", e);
                    return;
                }
            }
        }
    }

    // 多线程收集Frame
    pub fn start_to_collect(&mut self) {
        self.stop();
        let (tx, rx) = std::sync::mpsc::channel();
        {
            self.stop_signal.swap(false, Ordering::Acquire);
        }
        let tx = tx.clone();
        let only_en = self.only_en;
        std::thread::spawn(move || {
            let threads = datalink::interfaces()
                .into_iter()
                .filter(|item| only_en && item.name.starts_with("en"))
                .filter(|item| item.is_up() && !item.ips.is_empty())
                .flat_map(|item| {
                    println!("interface {}, start", &item.name);
                    let tx = tx.clone();
                    std::thread::Builder::new()
                        .name("collector_".to_owned() + &item.name)
                        .spawn(move || {
                            NetworkTraffic::get_packet(item, &|frame: Frame| {
                                tx.send(frame);
                            });
                        })
                })
                .collect::<Vec<JoinHandle<()>>>();
            println!("threads length {}", threads.len());
            for t in threads {
                t.join();
            }
            println!("all collector threads done");
        });

        let signal = Arc::clone(&self.stop_signal);
        let frames = Arc::clone(&self.frames);
        std::thread::spawn(|| {
            NetworkTraffic::receive(signal, frames, rx);
        });
    }

    fn get_packet(interface: NetworkInterface, handle_frame: &dyn Fn(Frame)) {
        let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("packetdump: unhandled channel type"),
            Err(e) => panic!("packetdump: unable to create channel: {}", e),
        };

        loop {
            match rx.next() {
                Ok(packet) => {
                    match analyze_packet(&interface, packet) {
                        None => {}
                        Some(frame) => {
                            handle_frame(frame);
                        }
                    }
                }
                Err(_e) => return,
            }
        }
    }
}