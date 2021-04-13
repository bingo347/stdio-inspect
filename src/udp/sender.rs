use crate::communicator::{Message, StreamKind, DATA_BUFFER_SIZE};
use std::net::SocketAddr;
use tokio::sync::broadcast::Receiver;
use tokio::time::{self, Duration};

const ACCUMULATOR_CAPACITY: usize = DATA_BUFFER_SIZE * 4;

pub async fn start_sender(addr: SocketAddr, mut rx: Receiver<Message>) {
    let mut acc = Accumulator {
        addr,
        last_stream: None,
        stream_data: Vec::with_capacity(ACCUMULATOR_CAPACITY),
    };
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = time::sleep(Duration::from_millis(500)) => {
                    acc.tick().await;
                },
                msg = rx.recv() => {
                    acc.push(msg.unwrap()).await;
                },
            }
        }
    });
    tokio::task::yield_now().await;
}

struct Accumulator {
    addr: SocketAddr,
    last_stream: Option<StreamKind>,
    stream_data: Vec<u8>,
}

impl Accumulator {
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
