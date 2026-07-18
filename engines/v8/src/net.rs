use crate::error::V8Error;

use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::io::{Read, Write};
use std::sync::Mutex;

pub struct TcpSocket {
    stream: Option<Mutex<TcpStream>>,
    local_addr: Option<String>,
    remote_addr: Option<String>,
}

impl TcpSocket {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, V8Error> {
        let stream = TcpStream::connect(addr).map_err(|e| V8Error::Internal(e.to_string()))?;
        let local = stream.local_addr().ok().map(|a| a.to_string());
        let remote = stream.peer_addr().ok().map(|a| a.to_string());
        Ok(Self {
            stream: Some(Mutex::new(stream)),
            local_addr: local,
            remote_addr: remote,
        })
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, V8Error> {
        if let Some(ref stream) = self.stream {
            stream.lock().unwrap().read(buf).map_err(|e| V8Error::Internal(e.to_string()))
        } else {
            Err(V8Error::Internal("not connected".into()))
        }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, V8Error> {
        if let Some(ref stream) = self.stream {
            stream.lock().unwrap().write(buf).map_err(|e| V8Error::Internal(e.to_string()))
        } else {
            Err(V8Error::Internal("not connected".into()))
        }
    }

    pub fn local_addr(&self) -> Option<&str> { self.local_addr.as_deref() }
    pub fn remote_addr(&self) -> Option<&str> { self.remote_addr.as_deref() }
}

pub struct TcpServer {
    listener: Option<Mutex<TcpListener>>,
}

impl TcpServer {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self, V8Error> {
        let listener = TcpListener::bind(addr).map_err(|e| V8Error::Internal(e.to_string()))?;
        Ok(Self { listener: Some(Mutex::new(listener)) })
    }

    pub fn accept(&self) -> Result<TcpSocket, V8Error> {
        if let Some(ref listener) = self.listener {
            let (stream, _) = listener.lock().unwrap().accept().map_err(|e| V8Error::Internal(e.to_string()))?;
            let local = stream.local_addr().ok().map(|a| a.to_string());
            let remote = stream.peer_addr().ok().map(|a| a.to_string());
            Ok(TcpSocket {
                stream: Some(Mutex::new(stream)),
                local_addr: local,
                remote_addr: remote,
            })
        } else {
            Err(V8Error::Internal("not bound".into()))
        }
    }
}

pub fn resolve_host(host: &str) -> Result<Vec<String>, V8Error> {
    let addrs: Vec<String> = (host, 0).to_socket_addrs()
        .map_err(|e| V8Error::Internal(e.to_string()))?
        .map(|a| a.ip().to_string())
        .collect();
    Ok(addrs)
}
