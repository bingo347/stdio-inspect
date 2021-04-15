use crate::communicator::{Message, StreamKind, DATA_BUFFER_SIZE};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::UdpSocket,
    sync::{
        broadcast::{error::RecvError, Receiver},
        watch, Mutex,
    },
    time::{self, Duration, Instant},
};

const ACCUMULATOR_CAPACITY: usize = DATA_BUFFER_SIZE * 4;
const MAX_UDP_BUFFER_SIZE: usize = 32768;
const TICK_DURATION: Duration = Duration::from_millis(500);

pub async fn start_sender(addr: SocketAddr, mut rx: Receiver<Message>) {
    let socket = UdpSocket::bind(("::", 0))
        .await
        .expect("Cannot bind send socket");
    let ticker = Ticker::new();
    let mut tick_rx = Arc::clone(&ticker).run();
    let acc = Arc::new(Mutex::new(Accumulator::new(socket, addr)));
    tokio::spawn({
        let acc = Arc::clone(&acc);
        async move {
            while tick_rx.changed().await.is_ok() {
                let mut acc = acc.lock().await;
                acc.tick().await;
            }
        }
    });
    tokio::spawn(async move {
        loop {
            let msg = match rx.recv().await {
                Ok(msg) => msg,
                Err(RecvError::Closed) => return,
                Err(RecvError::Lagged(_)) => continue,
            };
            ticker.reset().await;
            let mut acc = acc.lock().await;
            acc.push(msg).await;
        }
    });
    tokio::task::yield_now().await;
}

struct Ticker {
    next_tick_time: Mutex<Instant>,
}

impl Ticker {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            next_tick_time: Mutex::new(Instant::now() + TICK_DURATION),
        })
    }

    fn run(self: Arc<Self>) -> watch::Receiver<()> {
        let (tx, rx) = watch::channel(());
        tokio::spawn(async move {
            loop {
                let sleep_duration = self
                    .next_tick_time
                    .lock()
                    .await
                    .saturating_duration_since(Instant::now());
                time::sleep(sleep_duration).await;
                let mut next_tick_time = self.next_tick_time.lock().await;
                let check_duration = next_tick_time.checked_duration_since(Instant::now());
                if let Some(_) = check_duration {
                    *next_tick_time = Instant::now() + TICK_DURATION;
                    tx.send(()).unwrap();
                }
            }
        });
        rx
    }

    async fn reset(&self) {
        let mut next_tick_time = self.next_tick_time.lock().await;
        *next_tick_time = Instant::now() + TICK_DURATION;
    }
}

struct Accumulator {
    socket: UdpSocket,
    addr: SocketAddr,
    data: Vec<u8>,
    last_kind: Option<StreamKind>,
}

impl Accumulator {
    fn new(socket: UdpSocket, addr: SocketAddr) -> Self {
        let mut data = Vec::with_capacity(ACCUMULATOR_CAPACITY);
        data.push(255);
        Self {
            socket,
            addr,
            data,
            last_kind: None,
        }
    }

    async fn tick(&mut self) {
        let kind = match self.last_kind {
            Some(k) => k,
            None => return,
        };
        self.send_data(kind).await;
    }

    async fn push(&mut self, msg: Message) {
        let kind = msg.kind;
        let data = msg.get_data();
        match self.last_kind {
            Some(last_kind) if last_kind == kind => {
                self.data.extend_from_slice(data);
                if self.data.len() > MAX_UDP_BUFFER_SIZE {
                    self.send_data(kind).await;
                    self.last_kind = None;
                }
            }
            Some(last_kind) => {
                self.send_data(last_kind).await;
                self.last_kind = Some(kind);
            }
            None => {
                self.data.extend_from_slice(data);
                self.last_kind = Some(kind);
            }
        }
    }

    async fn send_data(&mut self, kind: StreamKind) {
        let mut data = {
            let mut new_data = Vec::with_capacity(ACCUMULATOR_CAPACITY);
            new_data.push(255);
            std::mem::replace(&mut self.data, new_data)
        };
        data[0] = match kind {
            StreamKind::Stdin => 0,
            StreamKind::Stdout => 1,
            StreamKind::Stderr => 2,
        };
        self.socket
            .send_to(&data, &self.addr)
            .await
            .expect("Cannot send data to socket");
    }
}
