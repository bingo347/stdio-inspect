use crate::communicator::{Message, StreamKind, DATA_BUFFER_SIZE};
use std::{
    convert::{TryFrom, TryInto},
    env::ArgsOs,
    ffi::OsString,
    io::ErrorKind,
    process::Stdio,
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    process::Command as ProcessCommand,
    sync::broadcast::Sender,
    task::JoinHandle,
};

#[derive(Debug)]
pub struct Command {
    command: OsString,
    args: ArgsOs,
}

impl Command {
    pub fn new<T: TryInto<Command>>(input: T) -> Option<Self> {
        match input.try_into() {
            Ok(command) => Some(command),
            _ => None,
        }
    }

    pub async fn run(self, sender: Sender<Message>) -> i32 {
        let mut child_process = ProcessCommand::new(self.command)
            .args(self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Cannot spawn child process");
        let stdin = child_process.stdin.take().unwrap();
        let stdout = child_process.stdout.take().unwrap();
        let stderr = child_process.stderr.take().unwrap();
        proxy_stream(io::stdin(), stdin, StreamKind::Stdin, &sender);
        proxy_stream(stdout, io::stdout(), StreamKind::Stdout, &sender);
        proxy_stream(stderr, io::stderr(), StreamKind::Stderr, &sender);
        let status = child_process
            .wait()
            .await
            .expect("Wait child process failed");
        status.code().unwrap_or(0)
    }
}

macro_rules! impl_try_from {
    ($t:ty => |$input:pat| $body:block) => {
        impl TryFrom<$t> for Command {
            type Error = ();
            fn try_from($input: $t) -> Result<Self, ()> {
                $body
            }
        }
    };
}
impl_try_from![ArgsOs => |mut args| {
    match args.next() {
        Some(command) => Ok(Self { command, args }),
        _ => Err(())
    }
}];
impl_try_from![(OsString, ArgsOs) => |(command, args)| {
    Ok(Self { command, args })
}];
impl_try_from![(String, ArgsOs) => |(command, args)| {
    let command = command.into();
    Ok(Self { command, args })
}];

fn proxy_stream<R, W>(
    mut r: R,
    mut w: W,
    kind: StreamKind,
    sender: &Sender<Message>,
) -> JoinHandle<()>
where
    R: AsyncReadExt + Unpin + Send + 'static,
    W: AsyncWriteExt + Unpin + Send + 'static,
{
    let sender = sender.clone();
    tokio::spawn(async move {
        let mut buffer = [0; DATA_BUFFER_SIZE];
        loop {
            match r.read(&mut buffer).await {
                Ok(0) => return,
                Ok(n) => {
                    let data_chunk = &buffer[..n];
                    sender
                        .send(Message::new(kind, data_chunk))
                        .unwrap_or_default();
                    w.write_all(data_chunk)
                        .await
                        .map_err(|e| (kind, e))
                        .expect("write stream error");
                }
                Err(e) => {
                    if e.kind() != ErrorKind::Interrupted {
                        panic!("read stream error: {:?}", (kind, e));
                    }
                }
            }
        }
    })
}
