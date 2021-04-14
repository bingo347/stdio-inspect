use crate::communicator::{Message, StreamKind, DATA_BUFFER_SIZE};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::UdpSocket,
    sync::{broadcast::Receiver, watch, Mutex},
    time::{self, Duration, Instant},
};

const ACCUMULATOR_CAPACITY: usize = DATA_BUFFER_SIZE * 4;
const MAX_UDP_BUFFER_SIZE: usize = 32768;
const TICK_DURATION: Duration = Duration::from_millis(500);

pub async fn start_sender(addr: SocketAddr, mut rx: Receiver<Message>) {
    let ticker = Ticker::new();
    let mut tick_rx = Arc::clone(&ticker).run();
    let acc = Arc::new(Mutex::new(Accumulator::new(addr)));
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
            let msg = rx.recv().await.unwrap();
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
    addr: SocketAddr,
    last_kind: Option<StreamKind>,
    data: Vec<u8>,
}

impl Accumulator {
    fn new(addr: SocketAddr) -> Self {
        let mut data = Vec::with_capacity(ACCUMULATOR_CAPACITY);
        data.push(255);
        Self {
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
        let data = self.replace_data_with_empty();
        send_data(&self.addr, kind, data).await;
    }

    async fn push(&mut self, msg: Message) {
        let kind = msg.kind;
        let data = msg.get_data();
        match self.last_kind {
            Some(last_kind) if last_kind == kind => {
                self.data.extend_from_slice(data);
                if self.data.len() > MAX_UDP_BUFFER_SIZE {
                    let data = self.replace_data_with_empty();
                    send_data(&self.addr, kind, data).await;
                }
            }
            Some(last_kind) => {
                let data = self.replace_data_with_empty();
                self.last_kind = Some(kind);
                send_data(&self.addr, last_kind, data).await;
            }
            None => {
                self.data.extend_from_slice(data);
                self.last_kind = Some(kind);
            }
        }
    }

    fn replace_data_with_empty(&mut self) -> Vec<u8> {
        let mut new_data = Vec::with_capacity(ACCUMULATOR_CAPACITY);
        new_data.push(255);
        self.last_kind = None;
        std::mem::replace(&mut self.data, new_data)
    }
}

async fn send_data(addr: &SocketAddr, kind: StreamKind, mut data: Vec<u8>) {
    data[0] = match kind {
        StreamKind::Stdin => 0,
        StreamKind::Stdout => 1,
        StreamKind::Stderr => 2,
    };
    let socket = UdpSocket::bind(("::", 0))
        .await
        .expect("Cannot bind send socket");
    socket
        .send_to(&data, addr)
        .await
        .expect("Cannot send data to socket");
}
