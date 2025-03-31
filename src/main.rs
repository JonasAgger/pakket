use std::sync::Arc;

use anyhow::{Context, bail};
use network::{Handler, ip::IpHandler, tcp::TcpHandler};
use proto::{NetworkBuffer, ProtocolBuffer, http::PackedHttpResp, ip::Ip, tcp::Tcp, udp::Udp};

mod application;
mod network;
pub mod oob_buffer;
mod proto;
mod utils;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let nic = create_nic()?;
    let nic = Arc::new(nic);

    let http_handler = network::http::HttpHandler::new(application::Api);

    let ip_layer = IpHandler {
        icmp: network::icmp::IcmpHandler,
        udp: network::udp::UdpHandler,
        tcp: TcpHandler::new(3000, http_handler, nic.clone()),
    };

    run_nic(nic, ip_layer)?;

    Ok(())
}

fn run_nic(nic: Arc<tun::Device>, mut ip_layer: IpHandler) -> anyhow::Result<()> {
    let mut buf = [0; 1500];

    loop {
        tracing::info!("RECEIVING");
        let bytes = nic.recv(&mut buf).context("Failed to recieve from nic")?;
        tracing::info!("RECV: {}", bytes);
        let ip_header = Ip::parse(&buf[..bytes])?;

        let out = ip_layer.handle(ip_header)?;

        if !out.is_empty() {
            _ = print(&out);
            // Sent will always be MTU size at least, it looks like.
            let sent = nic.send(&out).context("Failed to send to nic")?;
            tracing::info!("SENT: {}", sent);

            if sent < out.len() {
                bail!(
                    "Failed to send all bytes?? sent: {}, buflen: {}",
                    sent,
                    out.len()
                )
            }
        }
    }
    Ok(())
}

fn print(out: &NetworkBuffer) -> anyhow::Result<()> {
    let d = Ip::parse(out)?;

    tracing::info!("OUT: {}", d);
    match d.protocol() {
        proto::Protocol::TCP => {
            let dd = Tcp::parse(d)?;
            tracing::info!("OUT: {}", dd);

            if !dd.buf().is_empty() {
                let ddd = PackedHttpResp::parse(dd);
                tracing::info!("OUT: {}", ddd);
            }
        }
        proto::Protocol::UDP => {
            let dd = Udp::parse(d)?;
            tracing::info!("OUT: {}", dd);
        }
        other => return Ok(()),
    }

    Ok(())
}

fn create_nic() -> anyhow::Result<tun::Device> {
    let mut conf = tun::configure();
    conf.tun_name("utun9")
        .address((10, 0, 0, 1))
        // .broadcast((10, 0, 0, 255))
        .netmask((255, 255, 255, 0))
        // .destination((10, 0, 0, 1))
        .up();

    let nic = tun::create(&conf)?;
    Ok(nic)
}
