use std::net::{TcpListener, TcpStream, UdpSocket};
use std::io::{Read, Write};

pub struct JSCNet;

impl JSCNet {
    pub fn new() -> Self {
        Self
    }

    pub fn tcp_connect(&self, addr: &str) -> Result<TcpStream, String> {
        TcpStream::connect(addr).map_err(|e| format!("net.connect: {e}"))
    }

    pub fn tcp_bind(&self, addr: &str) -> Result<TcpListener, String> {
        TcpListener::bind(addr).map_err(|e| format!("net.bind: {e}"))
    }

    pub fn tcp_write(&self, mut stream: &TcpStream, data: &[u8]) -> Result<usize, String> {
        stream.write(data).map_err(|e| format!("net.write: {e}"))
    }

    pub fn tcp_read(&self, mut stream: &TcpStream, buf: &mut [u8]) -> Result<usize, String> {
        stream.read(buf).map_err(|e| format!("net.read: {e}"))
    }

    pub fn udp_bind(&self, addr: &str) -> Result<UdpSocket, String> {
        UdpSocket::bind(addr).map_err(|e| format!("net.udpBind: {e}"))
    }

    pub fn udp_send(&self, socket: &UdpSocket, buf: &[u8], addr: &str) -> Result<usize, String> {
        socket.send_to(buf, addr).map_err(|e| format!("net.udpSend: {e}"))
    }

    pub fn udp_recv(&self, socket: &UdpSocket, buf: &mut [u8]) -> Result<(usize, String), String> {
        let (len, addr) = socket.recv_from(buf).map_err(|e| format!("net.udpRecv: {e}"))?;
        Ok((len, addr.to_string()))
    }

    pub fn resolve_host(&self, host: &str) -> Result<Vec<std::net::IpAddr>, String> {
        use std::net::ToSocketAddrs;
        (host, 0).to_socket_addrs()
            .map_err(|e| format!("net.resolve: {e}"))?
            .map(|addr| Ok(addr.ip()))
            .collect()
    }
}

impl Default for JSCNet {
    fn default() -> Self {
        Self::new()
    }
}
