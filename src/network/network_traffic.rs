use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use nix::unistd::Uid;
use pnet::datalink;
use pnet::datalink::{DataLinkReceiver, DataLinkSender, NetworkInterface};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::EthernetPacket;
use serde::{Deserialize, Serialize};

use crate::network::analyze::Frame;
use crate::network::config::{InterfaceType, ProtocolType};

use super::analyze::handle_ethernet_frame;

pub struct NetworkTraffic {
    pub frames: Arc<Mutex<Vec<Frame>>>,
    pub another_frames: Arc<Mutex<Vec<Frame>>>,
    stop_signal: Arc<AtomicBool>,
    workers: Vec<(Sender<()>, Box<dyn DataLinkSender>, JoinHandle<()>)>,
    receiver: Vec<(Sender<()>, JoinHandle<()>)>,

    interface_types: Vec<InterfaceType>,
    // protocol_types: Vec<ProtocolType>
}

#[derive(Debug, Clone)]
#[repr(C)]
#[derive(Serialize, Deserialize)]
pub struct ProcessPacketLength {
    pub pid: u32,
    pub upload_length: usize,
    pub download_length: usize,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ProcessStatistics {
    // pub list_ptr: *const ProcessPacketLength,
    pub list: Vec<ProcessPacketLength>,

    pub total_upload: u64,
    pub total_download: u64,
    // pub elapse_millisecond: u64,
}

impl NetworkTraffic {
    pub fn new() -> Self {
        if !Uid::effective().is_root() {
            eprintln!("Not root user!");
        }
        NetworkTraffic {
            frames: Arc::new(Mutex::new(Vec::new())),
            another_frames: Arc::new(Mutex::new(Vec::new())),
            stop_signal: Arc::new(AtomicBool::new(true)),
            workers: Vec::new(),
            receiver: Vec::new(),
            interface_types: vec![InterfaceType::En],
            // protocol_types: vec![ProtocolType::Tcp, ProtocolType::Udp],
        }
    }

    // 从Vec中取数据
    pub fn take(&mut self) -> ProcessStatistics {
        let mut tmp = Vec::new();
        {
            let mut frames = self.frames.lock().ok().unwrap();

            tmp.resize(frames.len(), Frame {
                interface_name: "".to_string(),
                data_length: 0,
                is_upload: false,
                protocol: ProtocolType::Tcp,
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
        let port_process = crate::network::sys_info::get_port_process_map(&tmp);

        let mut total_upload: u64 = 0;
        let mut total_download: u64 = 0;
        for frame in tmp.iter() {
            if frame.is_upload {
                total_upload += frame.data_length as u64;
            } else {
                total_download += frame.data_length as u64;
            }
            match port_process.get(&frame.protocol_port()) {
                None => {
                    println!("unknown process");
                }
                Some(pid) => {
                    // 可能为0
                    let pid = *pid;
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

        ProcessStatistics {
            list,
            total_upload,
            total_download,
        }
    }

    // 停止收集，目前无法停止worker线程。主要是没有收到数据会阻塞在next函数，无法处理信号。
    pub fn stop(&mut self) {
        if self.workers.is_empty() && self.receiver.is_empty() {
            return;
        }
        println!("send stop signal");
        for (stop, _package, _handler) in &mut self.workers {
            let _ = stop.send(());
        }
        for (stop, _) in &mut self.receiver {
            let _ = stop.send(());
        }

        while let Some((_, handler)) = self.receiver.pop() {
            println!("shutdown {}", handler.thread().name().unwrap_or("unnamed thread"));
            let _ = handler.join();
        }
        self.stop_signal.swap(true, Ordering::Acquire);
    }

    // 将Frame推入Vec
    fn receive(frames: Arc<Mutex<Vec<Frame>>>, stop_rx: Receiver<()>, rx: Receiver<Frame>) {
        loop {
            if stop_rx.try_recv().is_ok() {
                return;
            };
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
        if !self.stop_signal.load(Ordering::Acquire) {
            println!("running");
            return;
        }
        self.stop();
        let (tx, rx) = std::sync::mpsc::channel();
        {
            self.stop_signal.swap(false, Ordering::Acquire);
        }

        self.receiver = vec![{
            let frames = Arc::clone(&self.frames);
            let (stop_tx, stop_rx) = std::sync::mpsc::channel();
            let thread = std::thread::Builder::new()
                .name("receiver".to_owned())
                .spawn(move || {
                    NetworkTraffic::receive(frames, stop_rx, rx);
                })
                .unwrap();
            (stop_tx, thread)
        }];


        let tx = tx.clone();
        let threads = datalink::interfaces()
            .into_iter()
            .filter(|item| {
                self.interface_types.iter().any(|interface_type| interface_type.filter(item))
            })
            .filter(|item| item.is_up() && !item.ips.is_empty())
            .filter_map(|item| {
                let config = Default::default();
                match datalink::channel(&item, config) {
                    Ok(Ethernet(tx, rx)) => {
                        Some((tx, rx, item))
                    }
                    Ok(_) => {
                        eprintln!("packetdump: unhandled channel type");
                        None
                    }
                    Err(e) => {
                        eprintln!("packetdump: unable to create channel: {}", e);
                        None
                    }
                }
            })
            .map(|(package_sender, package_receiver, item)| {
                println!("interface {}, start", &item.name);
                let tx = tx.clone();
                let (stop_tx, stop_rx) = mpsc::channel();
                let thread = std::thread::Builder::new()
                    .name(item.name.clone())
                    .spawn(move || {
                        NetworkTraffic::get_packet(item, stop_rx, package_receiver, tx);
                    });
                (stop_tx, package_sender, thread.unwrap())
            })
            .collect::<Vec<(Sender<()>, Box<dyn DataLinkSender>, JoinHandle<()>)>>();
        self.workers = threads;
    }

    fn get_packet(interface: NetworkInterface, stop_rx: Receiver<()>,
                  mut package_rx: Box<dyn DataLinkReceiver>,
                  frame_sender: Sender<Frame>) {
        loop {
            if stop_rx.try_recv().is_ok() {
                return;
            };
            match package_rx.next() {
                Ok(packet) => {
                    match handle_ethernet_frame(&interface, &EthernetPacket::new(packet).unwrap()) {
                        None => {}
                        Some(frame) => {
                            let send_result = frame_sender.send(frame);
                            if send_result.is_err() {
                                eprintln!("send error {}", send_result.err().unwrap());
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("receive package error {}", e);
                }
            }
        }
    }
}