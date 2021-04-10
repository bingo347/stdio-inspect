pub const DATA_BUFFER_SIZE: usize = 8;

#[derive(Debug, Clone, Copy)]
pub enum StreamKind {
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug, Clone)]
pub struct Message {
    kind: StreamKind,
    data: [u8; DATA_BUFFER_SIZE],
    len: usize,
}

impl Message {
    pub fn new(kind: StreamKind, data_chunk: &[u8]) -> Self {
        let len = data_chunk.len();
        assert!(len <= DATA_BUFFER_SIZE);
        let mut data = [0; DATA_BUFFER_SIZE];
        data[..len].copy_from_slice(data_chunk);
        Self { kind, data, len }
    }
}
