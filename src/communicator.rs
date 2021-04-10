const fn get_buffer_size() -> usize {
    const DEFAULT: usize = 8;
    let s = match option_env!("BUFFER_SIZE") {
        Some(s) => s,
        None => "",
    };
    let s = s.as_bytes();
    if s.len() == 0 {
        return DEFAULT;
    }
    let mut r = 0;
    let mut i = 0;
    while i < s.len() {
        let ch = s[i];
        if ch < b'0' || ch > b'9' {
            return DEFAULT;
        }
        r *= 10;
        r += (ch - b'0') as usize;
        i += 1;
    }
    r
}

pub const DATA_BUFFER_SIZE: usize = get_buffer_size();

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
