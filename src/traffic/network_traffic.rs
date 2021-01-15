use std::any::Any;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::error::Error;
use std::iter::Map;
use std::net::{IpAddr, Ipv4Addr};
use std::ops::BitXor;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvError};
use std::thread::{JoinHandle, Thread};
use std::time::Instant;

use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::datalink::NetworkInterface;
use pnet::transport::TransportProtocol::Ipv4;

use crate::traffic::analyze;
use crate::traffic::analyze::{analyze_packet, Frame};

#[derive(Debug, Clone)]
struct ProcessPacketLength {
    // 递归至非1的父进程ID
    pid: u32,
    upload_length: usize,
    download_length: usize,
}

#[derive(Debug)]
pub struct ProcessStatistics {
    length: usize,
    list: Vec<ProcessPacketLength>,
    // 本次收集的数据，使用的时间
    elapse_millisecond: u128,
}


pub struct NetworkTraffic {
    pub frames: Arc<Mutex<Vec<Frame>>>,
    pub another_frames: Arc<Mutex<Vec<Frame>>>,
    pub start_time: Instant,
    threads: Vec<JoinHandle<()>>,
    stop_signal: AtomicBool,
}

impl NetworkTraffic {
    pub fn new() -> Self {
        NetworkTraffic {
            frames: Arc::new(Mutex::new(Vec::new())),
            another_frames: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            threads: Vec::new(),
            stop_signal: AtomicBool::new(false),
        }
    }

    // 将收集的数据取出
    pub fn take(&mut self) -> Option<ProcessStatistics> {
        // 根据文档，想要早点解锁，这样应该可以。
        // An RAII guard is returned to allow scoped unlock of the lock. When
        // the guard goes out of scope, the mutex will be unlocked.
        let mut tmp = self.another_frames.lock().ok()?;
        let elapse = self.start_time.elapsed().as_millis();
        {
            let mut frames = self.frames.lock().ok()?;

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
            self.start_time = Instant::now();
        }
        println!("collect frames length: {}", tmp.len());
        // 将tmp中的数据集合一下
        let mut map = HashMap::<u32, ProcessPacketLength>::new();
        let port_process = crate::traffic::sys_info::get_port_process_map(&tmp);

        for frame in tmp.iter() {
            match port_process.get(&frame.local_port()) {
                None => {}
                Some(pid) => {
                    let mut upload: usize = 0;
                    let mut download: usize = 0;
                    if frame.is_upload {
                        upload = frame.data_length;
                    } else {
                        download = frame.data_length;
                    }
                    match map.get_mut(pid) {
                        None => {
                            map.insert(*pid, ProcessPacketLength {
                                pid: *pid,
                                upload_length: upload,
                                download_length: download,
                            });
                        }
                        Some(mut info) => {
                            info.download_length += download;
                            info.upload_length += upload;
                        }
                    }
                }
            }
        }

        let list = map.values().cloned().collect::<Vec<ProcessPacketLength>>();
        Some(ProcessStatistics {
            length: list.len(),
            list,
            elapse_millisecond: elapse,
        })

        // return Some(self.reduce(&tmp, elapse));
    }

    pub fn stop(&mut self) {
        println!("stop signal");
        *self.stop_signal.get_mut() = true
    }

    fn receive(&mut self, rx: Receiver<Frame>) {
        loop {
            {
                let signal = self.stop_signal.get_mut();
                if *signal {
                    println!("结束:{}", signal);
                    return;
                }
            }

            match rx.recv() {
                Ok(item) => {
                    match self.frames.lock().ok() {
                        None => {
                            println!("lock fail");
                        }
                        Some(mut v) => {
                            v.push(item);
                            println!("v length: {}", v.len());
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

    pub fn start_to_collect(&mut self) {
        // self.stop();
        let (tx, rx) = std::sync::mpsc::channel();
        self.receive(rx);
        // 在线，且有ip的网卡
        // N个网卡，N个线程处理
        let signal = self.stop_signal.get_mut();
        *signal = false;
        let threads = datalink::interfaces()
            .into_iter()
            .filter(|item| item.is_up() && !item.ips.is_empty())
            .filter(|item| item.name.eq("en1"))
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
        self.threads = threads;
        println!("当前线程数:{}", self.threads.len());
    }

    fn get_packet(interface: NetworkInterface, handle_frame: &dyn Fn(Frame)) {
        let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => panic!("packetdump: unhandled channel type: {}"),
            Err(e) => panic!("packetdump: unable to create channel: {}", e),
        };

        loop {
            match rx.next() {
                Ok(packet) => {
                    match analyze_packet(&interface, packet) {
                        None => {}
                        Some(frame) => {
                            // println!("{:?}", frame);
                            handle_frame(frame);
                        }
                    }
                }
                Err(e) => return,
            }
        }
    }
}