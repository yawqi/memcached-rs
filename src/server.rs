use crate::item::Item;
use crate::session::Session;

use color_eyre::Report;
use tokio::net::TcpListener;
use tracing::info;

use std::array;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    worker_channels: Vec<tokio::sync::mpsc::Sender<Arc<Session>>>,
    workers: Vec<tokio::task::JoinHandle<()>>,

    inner: Arc<Inner>,
}

impl Server {
    pub async fn setup(port: u16) -> Result<Self, Report> {
        let addr = "127.0.0.1:".to_string() + &port.to_string();
        let listener = TcpListener::bind(addr).await.unwrap();
        let worker_count = 16;

        let mut worker_channels = vec![];
        let mut workers = vec![];

        for _ in 0..worker_count {
            let (tx, mut rx) = tokio::sync::mpsc::channel(32);
            let handle = tokio::spawn(async move {
                while let Some(_) = rx.recv().await {
                    info!("helloworld1");
                    info!("helloworld2");
                }
            });

            worker_channels.push(tx);
            workers.push(handle);
        }

        Ok(Self {
            listener,
            worker_channels,
            workers,
            inner: Inner::setup(),
        })
    }

    pub async fn run(&self) {
        let mut counter = 0;
        loop {
            let (stream, client_addr) = self.listener.accept().await.unwrap();
            info!(%client_addr, "connected: ");

            let mut sessions = self.inner.sessions.lock().unwrap();
            let sess = sessions
                .entry(client_addr)
                .or_insert(Arc::new(Session::new(Arc::clone(&self.inner), stream)));

            self.worker_channels[counter]
                .send(Arc::clone(sess))
                .await
                .unwrap();

            counter = (counter + 1) % self.worker_channels.len();
        }
    }
}

#[derive(Debug)]
pub(crate) struct Inner {
    items: [Mutex<HashSet<Item>>; 4096],
    sessions: Mutex<HashMap<SocketAddr, Arc<Session>>>,
}

impl Inner {
    fn setup() -> Arc<Self> {
        let items = array::from_fn(|_| Mutex::new(HashSet::new()));
        Arc::new(Self {
            items,
            sessions: Mutex::new(HashMap::new()),
        })
    }
}
