use anyhow::{Context, bail};
use network::{Handler, ip::IpHandler, tcp::TcpHandler};
use proto::{
    ProtocolBuffer,
    http::{HttpReq, HttpResp},
    ip::Ip,
    tcp::Tcp,
};

mod network;
pub mod oob_buffer;
mod proto;
mod utils;

// todo: handle setting seq no when sending data
// fix not being able to write a TCP header after handling message by downstream consumer

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let nic = create_nic()?;
    let nic = std::sync::Arc::new(nic);

    let mut ip_layer = IpHandler {
        icmp: network::icmp::IcmpHandler,
        tcp: TcpHandler::new(3000, network::http::HttpHandler {}, nic.clone()),
    };

    let mut buf = [0; 1500];

    loop {
        tracing::info!("RECEIVING");
        let bytes = nic.recv(&mut buf).context("Failed to recieve from nic")?;
        tracing::info!("RECV: {}", bytes);
        let ip_header = Ip::parse(&buf[..bytes])?;

        let out = ip_layer.handle(ip_header)?;

        if !out.is_empty() {
            let d = Ip::parse(&out)?;

            tracing::info!("OUT: {}", d);
            let dd = Tcp::parse(d)?;
            tracing::info!("OUT: {}", dd);

            if !dd.buf().is_empty() {
                let ddd = HttpResp::parse(dd);
                tracing::info!("OUT: {}", ddd);
            }

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

fn create_nic() -> anyhow::Result<tun::Device> {
    let mut conf = tun::configure();
    conf.tun_name("utun9")
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();

    let nic = tun::create(&conf)?;
    Ok(nic)
}

fn create_nic2() -> anyhow::Result<std::sync::Arc<tun::Device>> {
    let mut conf = tun::configure();
    conf.tun_name("utun50")
        .address((10, 100, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 100, 0, 1))
        .up();

    let nic = tun::create(&conf)?;
    Ok(std::sync::Arc::new(nic))
}
