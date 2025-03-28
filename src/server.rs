use crate::item::Item;
use crate::session::Session;

use color_eyre::Report;
use tokio::net::TcpListener;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::Sender;
use tracing::info;

use std::array;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    worker_channels: Vec<Sender<Session>>,
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
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Session>(32);
            let handle = tokio::spawn(async move {
                while let Some(session) = rx.recv().await {
                    info!("hello world");
                    session.handle().await;
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
                .or_insert(Session::new(Arc::downgrade(&self.inner), stream));

            self.worker_channels[counter]
                .send(sess.clone())
                .await
                .unwrap();

            counter = (counter + 1) % self.worker_channels.len();
        }
    }

    pub async fn wait_for_shutdown(&self) {
        let mut intr_signal = signal(SignalKind::interrupt()).unwrap();
        let mut term_signal = signal(SignalKind::terminate()).unwrap();
        let mut quit_signal= signal(SignalKind::quit()).unwrap();

        select! {
            _ = intr_signal.recv() => {
                info!("Received SIGINT");
            }
            _ = term_signal.recv() => {
                info!("Received SIGTERM");
            }
            _ = quit_signal.recv() => {
                info!("Received SIGQUIT");
            }
        }

        // for worker in self.workers.drain(..) {
        //     worker.await.unwrap();
        // }
    }
}

#[derive(Debug)]
pub(crate) struct Inner {
    items: [Mutex<HashSet<Item>>; 4096],
    sessions: Mutex<HashMap<SocketAddr, Session>>,
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
