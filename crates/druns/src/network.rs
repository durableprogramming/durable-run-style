#[cfg(feature = "experimental-pcap")]
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo};
#[cfg(feature = "experimental-pcap")]
use pcap;
#[cfg(feature = "experimental-pcap")]
use etherparse;

#[cfg(feature = "experimental-pcap")]
pub fn get_process_connections(pid: u32) -> Vec<(String, u16, String, u16, String)> {
    let mut connections = Vec::new();
    if let Ok(sockets) = get_sockets_info(AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6, ProtocolFlags::TCP | ProtocolFlags::UDP) {
        for socket in sockets {
            if socket.associated_pids.contains(&pid) {
                match socket.protocol_socket_info {
                    ProtocolSocketInfo::Tcp(tcp_info) => {
                        let local_ip = tcp_info.local_addr.to_string();
                        let local_port = tcp_info.local_port;
                        let remote_ip = tcp_info.remote_addr.to_string();
                        let remote_port = tcp_info.remote_port;
                        connections.push((local_ip, local_port, remote_ip, remote_port, "tcp".to_string()));
                    }
                    ProtocolSocketInfo::Udp(_udp_info) => {
                        // UDP connections don't have remote addr in this crate, skip for now
                    }
                }
            }
        }
    }
    connections
}

#[cfg(feature = "experimental-pcap")]
pub fn build_bpf_filter(connections: &[(String, u16, String, u16, String)]) -> String {
    let mut filters = Vec::new();
    for (local_ip, local_port, remote_ip, remote_port, proto) in connections {
        let filter = format!(
            "({} and ((src host {} and src port {} and dst host {} and dst port {}) or (dst host {} and dst port {} and src host {} and src port {})))",
            proto, local_ip, local_port, remote_ip, remote_port, local_ip, local_port, remote_ip, remote_port
        );
        filters.push(filter);
    }
    if filters.is_empty() {
        "false".to_string() // No connections, capture nothing
    } else {
        filters.join(" or ")
    }
}

#[cfg(feature = "experimental-pcap")]
pub async fn monitor_network(pid: u32, tx: tokio::sync::mpsc::UnboundedSender<(u64, u64)>) {
    loop {
        // Get current connections for the process
        let connections = get_process_connections(pid);
        let filter = build_bpf_filter(&connections);

        // Capture packets for 1 second
        let mut total_sent = 0u64;
        let mut total_recv = 0u64;

        if let Ok(devices) = pcap::Device::list() {
            if let Some(device) = devices.into_iter().next() {
                if let Ok(cap) = pcap::Capture::from_device(device) {
                    if let Ok(mut cap) = cap.timeout(1000).open() {
                        if cap.filter(&filter, true).is_ok() {
                            let start_time = tokio::time::Instant::now();
                            while start_time.elapsed() < tokio::time::Duration::from_secs(1) {
                                match cap.next_packet() {
                                    Ok(packet) => {
                                        if let Ok(headers) = etherparse::PacketHeaders::from_ethernet_slice(&packet.data) {
                                            let payload_len = headers.payload.len() as u64;
                                            // Determine if sent or received
                                            if let Some(etherparse::IpHeader::Version4(ipv4, _)) = headers.ip {
                                                let src_ip = std::net::Ipv4Addr::from(ipv4.source);
                                                let is_sent = connections.iter().any(|(local_ip, _, _, _, _)| {
                                                    if let Ok(lip) = local_ip.parse::<std::net::Ipv4Addr>() {
                                                        lip == src_ip
                                                    } else {
                                                        false
                                                    }
                                                });
                                                if is_sent {
                                                    total_sent += payload_len;
                                                } else {
                                                    total_recv += payload_len;
                                                }
                                            } else if let Some(etherparse::IpHeader::Version6(_ipv6, _)) = headers.ip {
                                                // Handle IPv6, for now skip
                                            }
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    }
                }
            }
        }

        // Send the stats
        let _ = tx.send((total_recv, total_sent));

        // Wait before next capture
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
