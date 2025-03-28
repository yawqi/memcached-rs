use super::server;

use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio::io::AsyncReadExt;
use tracing::{info, instrument};

use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug)]
enum SessionState {
    Initial,
    ReadingRequest,
    WritingResponse,
    Finished,
}

#[derive(Clone, Copy, Debug)]
enum Protocol {
    Ascii,
}
#[derive(Debug)]
struct Inner {
    server: Weak<server::Inner>,
    stream: TcpStream,
    state: SessionState,
    protocol: Protocol,
    buffer: Vec<u8>,
    total_requests: u32,
}

impl Inner {
    pub fn setup(server: Weak<server::Inner>, stream: TcpStream) -> Self {
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
#[derive(Debug, Clone)]
pub struct Session {
    inner: Arc<Mutex<Inner>>,
}

impl Session {
    pub fn new(server: Weak<server::Inner>, stream: TcpStream) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::setup(server, stream))),
        }
    }

    #[instrument]
    pub(crate) async fn handle(&self) {
        info!(?self);
        loop {
            let mut session = self.inner.lock().await;
            info!(?session.state);
            match session.state {
                SessionState::Initial => {
                    {

                        session.state = SessionState::ReadingRequest;
                    }
                }

                SessionState::ReadingRequest => {
                    let mut buf = [0u8; 1024];
                    let n = session.stream.read(&mut buf).await.unwrap();
                    if n == 0 {
                        break;
                    }
                    session.buffer.extend_from_slice(&buf[..n]);
                    session.total_requests += 1;
                    session.state = SessionState::WritingResponse;
                }

                SessionState::WritingResponse => {
                    let buffer= session.buffer.drain(..).collect::<Vec<u8>>();
                    session.stream.write(buffer.as_slice()).await.unwrap();
                    session.state = SessionState::Finished;
                }

                SessionState::Finished => {
                    break;
                }
            }
        }
    }
}