use std::sync::Arc;

use crate::{
    network::Handler,
    proto::{
        NetworkBuffer, ProtocolBuffer,
        ip::{Ip, IpHeaderWriter},
        tcp::{Tcp, TcpControl, TcpHeaderWriter},
    },
};

pub struct TcpState {
    state: State,
    sequence: TcpSequences,
    requires_ack: bool,
    nic: Arc<tun::Device>,
}

impl TcpState {
    pub fn new(nic: Arc<tun::Device>) -> Self {
        Self {
            state: State::Listen,
            sequence: Default::default(),
            requires_ack: false,
            nic,
        }
    }

    pub fn send(&mut self, data: NetworkBuffer, msg: Tcp<Ip<'_>>) -> NetworkBuffer {
        let buf = TcpHeaderWriter::new(
            msg.destination_port(),
            msg.source_port(),
            self.sequence.server_sequence,
            self.sequence.client_sequence,
        );

        self.sequence.server_sequence += data.len() as u32;

        let buf = if !data.is_empty() {
            buf.set(TcpControl::PSH).data(data)
        } else {
            buf
        };

        let buf = if self.requires_ack {
            self.requires_ack = false;
            buf.set(TcpControl::ACK)
        } else {
            buf
        };

        buf.calc_checksum(msg.inner()).to_buf()
    }

    fn send_fin(&self, msg: Tcp<Ip<'_>>) {
        let tcp = TcpHeaderWriter::new(
            msg.destination_port(),
            msg.source_port(),
            self.sequence.server_sequence,
            self.sequence.client_sequence,
        )
        .set(TcpControl::FIN)
        .calc_checksum(msg.inner())
        .to_buf();
        let ip = IpHeaderWriter::new(
            msg.inner().destination(),
            msg.inner().source(),
            crate::proto::Protocol::TCP,
            64,
            tcp,
        )
        .to_buf();

        let nic = self.nic.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            tracing::info!("SENDING FIN");
            nic.send(&ip).unwrap();
        });
    }
}

pub enum TcpControlMessage<'a> {
    None(Tcp<Ip<'a>>),
    Intercepted(NetworkBuffer),
    Closed,
}

#[derive(Default, Debug)]
pub struct TcpSequences {
    client_sequence: u32,
    server_sequence: u32,
}

impl<'a> Handler<Tcp<Ip<'a>>> for TcpState {
    type Retrun = TcpControlMessage<'a>;
    fn handle(&mut self, msg: Tcp<Ip<'a>>) -> anyhow::Result<Self::Retrun> {
        let tcp_control = msg.control();

        match self.state {
            State::Listen if tcp_control.contains(TcpControl::SYN) => {
                tracing::info!("Received SYN while listening, Sending Syn/Ack");
                self.state = State::SynRecv;
                self.sequence.client_sequence = msg.sequence_number() + 1;
                self.sequence.server_sequence = 0;

                let header = TcpHeaderWriter::new(
                    msg.destination_port(),
                    msg.source_port(),
                    self.sequence.server_sequence,
                    self.sequence.client_sequence,
                )
                .set(TcpControl::SYN | TcpControl::ACK)
                .calc_checksum(msg.inner());

                self.sequence.server_sequence += 1;

                Ok(TcpControlMessage::Intercepted(header.to_buf()))
            }
            State::SynRecv if tcp_control.contains(TcpControl::ACK) => {
                tracing::info!("Received ACK of Syn, moving to established");

                if msg.sequence_number() == self.sequence.client_sequence
                    && msg.ack_number() == self.sequence.server_sequence
                {
                    tracing::info!("It acked!");
                }

                self.state = State::Established;

                Ok(TcpControlMessage::Intercepted(NetworkBuffer::empty()))
            }
            State::Established if tcp_control.contains(TcpControl::FIN) => {
                tracing::info!("RECEIVED FIN");
                self.sequence.client_sequence += 1;

                let buf = TcpHeaderWriter::new(
                    msg.destination_port(),
                    msg.source_port(),
                    self.sequence.server_sequence,
                    self.sequence.client_sequence,
                )
                .set(TcpControl::ACK)
                .calc_checksum(msg.inner())
                .to_buf();

                self.state = State::LastAck;
                self.send_fin(msg);
                Ok(TcpControlMessage::Intercepted(buf))
            }
            State::Established => {
                let data_length = msg.buf().len();

                if data_length > 0 {
                    self.requires_ack = true;
                    self.sequence.client_sequence += (data_length) as u32;
                    Ok(TcpControlMessage::None(msg))
                } else {
                    Ok(TcpControlMessage::Intercepted(NetworkBuffer::empty()))
                }
            }
            State::LastAck => {
                tracing::info!("Received TCP Frame while in LastAck, should be close");

                if msg.sequence_number() == (self.sequence.client_sequence)
                    && msg.ack_number() == (self.sequence.server_sequence + 1)
                {
                    tracing::info!("It Closed!");
                }
                Ok(TcpControlMessage::Closed)
            }
            State::Listen | State::SynRecv => {
                tracing::info!("UNEXPECTED STATE");
                // std::thread::sleep(std::time::Duration::from_secs(30));
                Ok(TcpControlMessage::Closed)

                // other => bail!(
                //     "Received TcpControl {:?} when in state: {:?}",
                //     tcp_control,
                //     other
                // ),
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Listen,
    SynRecv,
    Established,
    LastAck,
}

impl Default for State {
    fn default() -> Self {
        Self::Listen
    }
}
