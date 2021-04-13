use crate::communicator::{Message, StreamKind, DATA_BUFFER_SIZE};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    sync::{broadcast::Receiver, watch, Mutex},
    time::{self, Duration, Instant},
};

const ACCUMULATOR_CAPACITY: usize = DATA_BUFFER_SIZE * 4;
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
                time::sleep(Duration::from_millis(10)).await;
                let mut next_tick_time = self.next_tick_time.lock().await;
                if let Some(_) = next_tick_time.checked_duration_since(Instant::now()) {
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
    last_stream: Option<StreamKind>,
    stream_data: Vec<u8>,
}

impl Accumulator {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            last_stream: None,
            stream_data: Vec::with_capacity(ACCUMULATOR_CAPACITY),
        }
    }

    async fn tick(&mut self) {
        let kind = match self.last_stream {
            Some(k) => k,
            None => return,
        };
        self.last_stream = None;
        let data = std::mem::replace(
            &mut self.stream_data,
            Vec::with_capacity(ACCUMULATOR_CAPACITY),
        );
        send_data(&self.addr, kind, data).await;
    }

    async fn push(&mut self, _msg: Message) {
        todo!()
    }
}

async fn send_data(_addr: &SocketAddr, _kind: StreamKind, _data: Vec<u8>) {
    todo!();
}
