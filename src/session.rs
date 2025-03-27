use super::server;

use tokio::net::TcpStream;

use std::sync::Arc;

#[derive(Debug)]
pub struct Session {
    server: Arc<server::Inner>,
    stream: TcpStream,
    state: SessionState,
    protocol: Protocol,
    buffer: Vec<u8>,

    total_requests: u32,
}

impl Session {
    pub fn new(server: Arc<server::Inner>, stream: TcpStream) -> Self {
        Self {
            server,
            stream,
            state: SessionState::Initial,
            protocol: Protocol::Ascii,
            buffer: vec![],
            total_requests: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum SessionState {
    Initial,
    ReadingRequest,
    WritingResponse,
}

#[derive(Clone, Copy, Debug)]
enum Protocol {
    Ascii,
}
