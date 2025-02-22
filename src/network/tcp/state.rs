use anyhow::bail;

use crate::{
    network::Handler,
    proto::{
        ip::Ip,
        tcp::{Tcp, TcpControl, TcpHeaderWriter},
        NetworkBuffer,
    },
};

#[derive(Debug, Default)]
pub struct TcpState {
    state: State,
    sequence: TcpSequences,
}

pub enum TcpControlMessage<'a> {
    None(Tcp<Ip<'a>>, TcpHeaderWriter),
    Intercepted(NetworkBuffer),
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
                println!("Received SYN while listening, Sending Syn/Ack");
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

                Ok(TcpControlMessage::Intercepted(header.to_buf()))
            }
            State::SynRecv if tcp_control.contains(TcpControl::ACK) => {
                println!("Received ACK of Syn, moving to established");

                if msg.sequence_number() == (self.sequence.client_sequence)
                    && msg.ack_number() == (self.sequence.server_sequence + 1)
                {
                    println!("It acked!");
                }

                self.state = State::Established;

                Ok(TcpControlMessage::Intercepted(NetworkBuffer::empty()))
            }
            State::Established => {
                self.sequence.client_sequence += 1;
                self.sequence.server_sequence += 1;
                let header = TcpHeaderWriter::new(
                    msg.destination_port(),
                    msg.source_port(),
                    self.sequence.server_sequence,
                    self.sequence.client_sequence,
                )
                .set(TcpControl::ACK);

                Ok(TcpControlMessage::None(msg, header))
            }
            other => bail!(
                "Received TcpControl {:?} when in state: {:?}",
                tcp_control,
                other
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum State {
    Listen,
    SynRecv,
    Established,
}

impl Default for State {
    fn default() -> Self {
        Self::Listen
    }
}
