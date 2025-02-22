use anyhow::{bail, Context};
use network::{ip::IpHandler, tcp::TcpHandler, Handler};
use proto::{ip::Ip, tcp::Tcp};

mod network;
mod proto;
mod utils;

fn main() -> anyhow::Result<()> {
    let mut ip_layer = IpHandler {
        icmp: network::icmp::IcmpHandler,
        tcp: TcpHandler::new(3000, network::http::HttpHandler {}),
    };

    let nic = create_nic()?;

    let mut buf = [0; 1500];

    loop {
        let bytes = nic.recv(&mut buf).context("Failed to recieve from nic")?;
        let ip_header = Ip::parse(&buf[..bytes])?;

        let out = ip_layer.handle(ip_header)?;

        if !out.is_empty() {
            let d = Ip::parse(&out)?;

            println!("OUT: {}", d);
            let dd = Tcp::parse(d)?;
            println!("OUT: {}", dd);

            // Sent will always be MTU size at least, it looks like.
            let sent = nic.send(&out).context("Failed to send to nic")?;

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
