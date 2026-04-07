use crate::{Datagram, MulticastSocket};

pub struct MulticastReceiver<'a> {
    socket: &'a MulticastSocket,
    buf_size: usize,
}

impl<'a> MulticastReceiver<'a> {
    pub fn new(socket: &'a MulticastSocket, buf_size: usize) -> Self {
        Self { socket, buf_size }
    }

    pub async fn recv(&self) -> std::io::Result<Datagram> {
        self.socket.recv_datagram(self.buf_size).await
    }
}
