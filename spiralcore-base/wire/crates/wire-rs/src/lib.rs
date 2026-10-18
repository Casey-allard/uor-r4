//! Rust binding for the UOR wire framing: WireSocket client and WireListener server.
//!
//! Wire format is unchanged and compatible with tools/wire_adapter.py and tools/wire_socket.py:
//!   handshake  MAGIC (UORADP01), u8 route length, route name, MAGIC reply
//!   stream     u32 header + payload per direction; bit31 marks a raw frame; zero length = EOS
//!   mode       first chunk of each direction picks raw (>= 4 KiB, incompressible) or streaming zlib
use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress, Status};
use std::io::{IoSlice, Read, Write};
use std::net::{TcpListener, TcpStream};

pub const MAGIC: &[u8; 8] = b"UORADP01";
const HEADER: usize = 4;
const RAW_FLAG: u32 = 1 << 31;
const CHUNK: usize = 65536;

fn chunk_size() -> usize {
    static CHUNK_ONCE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *CHUNK_ONCE.get_or_init(|| {
        std::env::var("WIRE_CHUNK")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(CHUNK)
            .clamp(4096, 4 * 1024 * 1024)
    })
}
const LEVEL: u32 = 6;

pub(crate) fn level() -> u32 {
    static LEVEL_ONCE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *LEVEL_ONCE.get_or_init(|| {
        std::env::var("WIRE_LEVEL")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(LEVEL)
            .clamp(0, 9)
    })
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub plain_in: u64,
    pub plain_out: u64,
    pub wire_in: u64,
    pub wire_out: u64,
    pub connections: u64,
}

impl Stats {
    pub fn ratio(&self) -> f64 {
        let plain = (self.plain_in + self.plain_out) as f64;
        let wire = (self.wire_in + self.wire_out) as f64;
        if plain > 0.0 {
            wire / plain
        } else {
            0.0
        }
    }
}

enum SendMode {
    Undecided,
    Raw,
    Compressed(Compress),
}

enum RecvMode {
    Raw,
    Compressed(Decompress),
}

pub mod mux;

pub struct WireSocket {
    stream: TcpStream,
    stats: Stats,
    send_mode: SendMode,
    scratch: Vec<u8>,
    recv_mode: Option<RecvMode>,
    recv_buf: Vec<u8>,
    eof: bool,
    eof_sent: bool,
}

pub(crate) fn compress_probe(data: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Compression::new(level()));
    encoder.write_all(data).expect("probe write");
    encoder.finish().expect("probe finish")
}

fn write_frame_parts(stream: &mut TcpStream, header: u32, payload: &[u8]) -> std::io::Result<()> {
    let head = header.to_be_bytes();
    let total = HEADER + payload.len();
    let mut written = 0usize;
    while written < total {
        let (head_slice, payload_slice) = if written < HEADER {
            (&head[written..], payload)
        } else {
            (&head[HEADER..], &payload[written - HEADER..])
        };
        let buffers = [IoSlice::new(head_slice), IoSlice::new(payload_slice)];
        let sent = stream.write_vectored(&buffers)?;
        if sent == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "write zero bytes",
            ));
        }
        written += sent;
    }
    Ok(())
}

fn compress_chunk_into(compressor: &mut Compress, data: &[u8], scratch: &mut Vec<u8>) -> usize {
    let base_in = compressor.total_in() as usize;
    let base_out = compressor.total_out() as usize;
    let needed = data.len() * 2 + 4096;
    if scratch.len() < needed {
        scratch.resize(needed, 0);
    }
    loop {
        let consumed = compressor.total_in() as usize - base_in;
        let produced = compressor.total_out() as usize - base_out;
        if consumed >= data.len() {
            return produced;
        }
        let status = compressor
            .compress(
                &data[consumed..],
                &mut scratch[produced..],
                FlushCompress::Sync,
            )
            .expect("compress");
        let now_in = compressor.total_in() as usize - base_in;
        let now_out = compressor.total_out() as usize - base_out;
        if status == Status::BufError && now_in == consumed && now_out == produced {
            return now_out;
        }
        if now_out >= scratch.len() {
            scratch.resize(scratch.len() * 2, 0);
        }
    }
}

pub(crate) fn compress_chunk(compressor: &mut Compress, data: &[u8]) -> Vec<u8> {
    let base_in = compressor.total_in() as usize;
    let base_out = compressor.total_out() as usize;
    let mut out = vec![0u8; data.len() * 2 + 4096];
    let mut produced;
    loop {
        let consumed = compressor.total_in() as usize - base_in;
        produced = compressor.total_out() as usize - base_out;
        if consumed >= data.len() {
            break;
        }
        if produced >= out.len() {
            let extra = out.len();
            out.resize(out.len() + extra, 0);
        }
        let status = compressor
            .compress(&data[consumed..], &mut out[produced..], FlushCompress::Sync)
            .expect("compress");
        let now_in = compressor.total_in() as usize - base_in;
        let now_out = compressor.total_out() as usize - base_out;
        if status == Status::BufError && now_in == consumed && now_out == produced {
            break;
        }
    }
    produced = compressor.total_out() as usize - base_out;
    out.truncate(produced);
    out
}

pub(crate) fn decompress_chunk(decompressor: &mut Decompress, payload: &[u8]) -> Vec<u8> {
    let base_in = decompressor.total_in() as usize;
    let base_out = decompressor.total_out() as usize;
    let mut out = vec![0u8; payload.len() * 64 + CHUNK];
    loop {
        let consumed = decompressor.total_in() as usize - base_in;
        let produced = decompressor.total_out() as usize - base_out;
        if consumed >= payload.len() {
            break;
        }
        if produced >= out.len() {
            let extra = out.len();
            out.resize(out.len() + extra, 0);
        }
        let status = decompressor
            .decompress(
                &payload[consumed..],
                &mut out[produced..],
                FlushDecompress::Sync,
            )
            .expect("decompress");
        let now_in = decompressor.total_in() as usize - base_in;
        let now_out = decompressor.total_out() as usize - base_out;
        if now_in == consumed && now_out == produced {
            break;
        }
        if status == Status::BufError && now_out == produced {
            break;
        }
    }
    let produced = decompressor.total_out() as usize - base_out;
    out.truncate(produced);
    out
}

impl WireSocket {
    fn wrap(stream: TcpStream) -> Self {
        WireSocket {
            stream,
            stats: Stats::default(),
            send_mode: SendMode::Undecided,
            scratch: Vec::new(),
            recv_mode: None,
            recv_buf: Vec::new(),
            eof: false,
            eof_sent: false,
        }
    }

    /// Connect to a wire endpoint (adapter gateway, WireListener, or routed hub) and handshake.
    pub fn connect(addr: &str, route: &str) -> std::io::Result<Self> {
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        let mut frame = Vec::with_capacity(MAGIC.len() + 1 + route.len());
        frame.extend_from_slice(MAGIC);
        frame.push(route.len().min(255) as u8);
        frame.extend_from_slice(&route.as_bytes()[..route.len().min(255)]);
        stream.write_all(&frame)?;
        let mut reply = [0u8; 8];
        stream.read_exact(&mut reply)?;
        if &reply != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "wire handshake failed",
            ));
        }
        let mut socket = Self::wrap(stream);
        socket.stats.connections = 1;
        Ok(socket)
    }

    fn send_frame(&mut self, header: u32, payload: &[u8]) -> std::io::Result<()> {
        let mut frame = Vec::with_capacity(HEADER + payload.len());
        frame.extend_from_slice(&header.to_be_bytes());
        frame.extend_from_slice(payload);
        self.stream.write_all(&frame)?;
        self.stats.wire_out += frame.len() as u64;
        Ok(())
    }

    pub fn send_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        if self.eof_sent {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "send after shutdown",
            ));
        }
        for piece in data.chunks(chunk_size()) {
            if matches!(self.send_mode, SendMode::Undecided) {
                let probe = compress_probe(piece);
                if piece.len() >= 4096 && probe.len() >= piece.len() {
                    self.send_mode = SendMode::Raw;
                } else {
                    self.send_mode =
                        SendMode::Compressed(Compress::new(Compression::new(level()), true));
                }
            }
            match &mut self.send_mode {
                SendMode::Raw => {
                    write_frame_parts(&mut self.stream, RAW_FLAG | piece.len() as u32, piece)?;
                    self.stats.wire_out += (HEADER + piece.len()) as u64;
                }
                SendMode::Compressed(compressor) => {
                    let produced = compress_chunk_into(compressor, piece, &mut self.scratch);
                    write_frame_parts(
                        &mut self.stream,
                        produced as u32,
                        &self.scratch[..produced],
                    )?;
                    self.stats.wire_out += (HEADER + produced) as u64;
                }
                SendMode::Undecided => unreachable!(),
            }
            self.stats.plain_in += piece.len() as u64;
        }
        Ok(())
    }

    fn read_frame(&mut self) -> std::io::Result<()> {
        let mut header = [0u8; HEADER];
        match self.stream.read_exact(&mut header) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.eof = true;
                return Ok(());
            }
            Err(error) => return Err(error),
        }
        let word = u32::from_be_bytes(header);
        let raw = word & RAW_FLAG != 0;
        let size = (word & !RAW_FLAG) as usize;
        self.stats.wire_in += (HEADER + size) as u64;
        if size == 0 {
            self.eof = true;
            return Ok(());
        }
        if self.recv_mode.is_none() {
            self.recv_mode = Some(if raw {
                RecvMode::Raw
            } else {
                RecvMode::Compressed(Decompress::new(true))
            });
        }
        match &mut self.recv_mode {
            Some(RecvMode::Raw) => {
                let before = self.recv_buf.len();
                self.recv_buf.resize(before + size, 0);
                self.stream.read_exact(&mut self.recv_buf[before..])?;
                self.stats.plain_out += size as u64;
            }
            Some(RecvMode::Compressed(decompressor)) => {
                let mut payload = vec![0u8; size];
                self.stream.read_exact(&mut payload)?;
                let base_in = decompressor.total_in() as usize;
                let base_out = decompressor.total_out() as usize;
                let before = self.recv_buf.len();
                loop {
                    let consumed = decompressor.total_in() as usize - base_in;
                    if consumed >= payload.len() {
                        break;
                    }
                    let produced = decompressor.total_out() as usize - base_out;
                    let spare = 65536usize;
                    self.recv_buf.resize(before + produced + spare, 0);
                    let status = decompressor
                        .decompress(
                            &payload[consumed..],
                            &mut self.recv_buf[before + produced..],
                            FlushDecompress::Sync,
                        )
                        .map_err(|error| {
                            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
                        })?;
                    let now_produced = decompressor.total_out() as usize - base_out;
                    self.recv_buf.truncate(before + now_produced);
                    if now_produced == produced || status == Status::BufError {
                        break;
                    }
                }
                let produced = decompressor.total_out() as usize - base_out;
                self.recv_buf.truncate(before + produced);
                self.stats.plain_out += produced as u64;
            }
            None => unreachable!(),
        }
        Ok(())
    }

    pub fn recv(&mut self, max: usize) -> std::io::Result<Vec<u8>> {
        while self.recv_buf.is_empty() && !self.eof {
            self.read_frame()?;
        }
        if self.recv_buf.len() <= max {
            return Ok(std::mem::take(&mut self.recv_buf));
        }
        let take = max;
        let data = self.recv_buf.drain(..take).collect();
        Ok(data)
    }

    pub fn recv_into(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        while self.recv_buf.is_empty() && !self.eof {
            self.read_frame()?;
        }
        let take = self.recv_buf.len().min(out.len());
        out[..take].copy_from_slice(&self.recv_buf[..take]);
        self.recv_buf.drain(..take);
        Ok(take)
    }

    pub fn recv_all(&mut self) -> std::io::Result<Vec<u8>> {
        let mut all = Vec::new();
        loop {
            let chunk = self.recv(CHUNK)?;
            if chunk.is_empty() {
                return Ok(all);
            }
            all.extend_from_slice(&chunk);
        }
    }

    pub fn shutdown(&mut self) -> std::io::Result<()> {
        if !self.eof_sent {
            self.send_frame(0, &[])?;
            self.eof_sent = true;
            self.stream.shutdown(std::net::Shutdown::Write)?;
        }
        Ok(())
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }
}

pub struct WireListener {
    listener: TcpListener,
}

impl WireListener {
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        Ok(WireListener {
            listener: TcpListener::bind(addr)?,
        })
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Accept one wire connection: reads and validates the handshake, replies MAGIC, returns (socket, route).
    pub fn accept(&self) -> std::io::Result<(WireSocket, String)> {
        let (mut stream, _) = self.listener.accept()?;
        stream.set_nodelay(true)?;
        let mut magic = [0u8; 8];
        stream.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "bad wire magic",
            ));
        }
        let mut length = [0u8; 1];
        stream.read_exact(&mut length)?;
        let mut route = vec![0u8; length[0] as usize];
        if !route.is_empty() {
            stream.read_exact(&mut route)?;
        }
        stream.write_all(MAGIC)?;
        let route = String::from_utf8_lossy(&route).to_string();
        let mut socket = WireSocket::wrap(stream);
        socket.stats.connections = 1;
        Ok((socket, route))
    }
}

/// Write half of a split wire connection (framing + compression state only).
pub struct WireWriter {
    stream: TcpStream,
    send_mode: SendMode,
    scratch: Vec<u8>,
    eof_sent: bool,
    stats: Stats,
}

impl WireWriter {
    fn send_frame(&mut self, header: u32, payload: &[u8]) -> std::io::Result<()> {
        write_frame_parts(&mut self.stream, header, payload)?;
        self.stats.wire_out += (HEADER + payload.len()) as u64;
        Ok(())
    }

    pub fn send_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        if self.eof_sent {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "send after shutdown",
            ));
        }
        for piece in data.chunks(chunk_size()) {
            if matches!(self.send_mode, SendMode::Undecided) {
                let sample = if piece.len() > 65536 {
                    &piece[..65536]
                } else {
                    piece
                };
                let probe = compress_probe(sample);
                if sample.len() >= 4096 && probe.len() >= sample.len() {
                    self.send_mode = SendMode::Raw;
                } else {
                    self.send_mode =
                        SendMode::Compressed(Compress::new(Compression::new(level()), true));
                }
            }
            match &mut self.send_mode {
                SendMode::Raw => {
                    write_frame_parts(&mut self.stream, RAW_FLAG | piece.len() as u32, piece)?;
                    self.stats.wire_out += (HEADER + piece.len()) as u64;
                }
                SendMode::Compressed(compressor) => {
                    let produced = compress_chunk_into(compressor, piece, &mut self.scratch);
                    write_frame_parts(
                        &mut self.stream,
                        produced as u32,
                        &self.scratch[..produced],
                    )?;
                    self.stats.wire_out += (HEADER + produced) as u64;
                }
                SendMode::Undecided => unreachable!(),
            }
            self.stats.plain_in += piece.len() as u64;
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> std::io::Result<()> {
        if !self.eof_sent {
            self.send_frame(0, &[])?;
            self.eof_sent = true;
            self.stream.shutdown(std::net::Shutdown::Write)?;
        }
        Ok(())
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }
}

/// Read half of a split wire connection (frame parsing + decompression state only).
pub struct WireReader {
    stream: TcpStream,
    recv_mode: Option<RecvMode>,
    recv_buf: Vec<u8>,
    eof: bool,
    stats: Stats,
}

impl WireReader {
    fn read_frame(&mut self) -> std::io::Result<()> {
        let mut header = [0u8; HEADER];
        match self.stream.read_exact(&mut header) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.eof = true;
                return Ok(());
            }
            Err(error) => return Err(error),
        }
        let word = u32::from_be_bytes(header);
        let raw = word & RAW_FLAG != 0;
        let size = (word & !RAW_FLAG) as usize;
        self.stats.wire_in += (HEADER + size) as u64;
        if size == 0 {
            self.eof = true;
            return Ok(());
        }
        if self.recv_mode.is_none() {
            self.recv_mode = Some(if raw {
                RecvMode::Raw
            } else {
                RecvMode::Compressed(Decompress::new(true))
            });
        }
        match &mut self.recv_mode {
            Some(RecvMode::Raw) => {
                let before = self.recv_buf.len();
                self.recv_buf.resize(before + size, 0);
                self.stream.read_exact(&mut self.recv_buf[before..])?;
                self.stats.plain_out += size as u64;
            }
            Some(RecvMode::Compressed(decompressor)) => {
                let mut payload = vec![0u8; size];
                self.stream.read_exact(&mut payload)?;
                let base_in = decompressor.total_in() as usize;
                let base_out = decompressor.total_out() as usize;
                let before = self.recv_buf.len();
                loop {
                    let consumed = decompressor.total_in() as usize - base_in;
                    if consumed >= payload.len() {
                        break;
                    }
                    let produced = decompressor.total_out() as usize - base_out;
                    let spare = 65536usize;
                    self.recv_buf.resize(before + produced + spare, 0);
                    let status = decompressor
                        .decompress(
                            &payload[consumed..],
                            &mut self.recv_buf[before + produced..],
                            FlushDecompress::Sync,
                        )
                        .map_err(|error| {
                            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
                        })?;
                    let now_produced = decompressor.total_out() as usize - base_out;
                    self.recv_buf.truncate(before + now_produced);
                    if now_produced == produced || status == Status::BufError {
                        break;
                    }
                }
                let produced = decompressor.total_out() as usize - base_out;
                self.recv_buf.truncate(before + produced);
                self.stats.plain_out += produced as u64;
            }
            None => unreachable!(),
        }
        Ok(())
    }

    pub fn recv(&mut self, max: usize) -> std::io::Result<Vec<u8>> {
        while self.recv_buf.is_empty() && !self.eof {
            self.read_frame()?;
        }
        if self.recv_buf.len() <= max {
            return Ok(std::mem::take(&mut self.recv_buf));
        }
        let take = max;
        Ok(self.recv_buf.drain(..take).collect())
    }

    pub fn recv_into(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        while self.recv_buf.is_empty() && !self.eof {
            self.read_frame()?;
        }
        let take = self.recv_buf.len().min(out.len());
        out[..take].copy_from_slice(&self.recv_buf[..take]);
        self.recv_buf.drain(..take);
        Ok(take)
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }
}

/// Split a connected WireSocket into independent read/write halves (one cloned TcpStream each).
pub fn split(socket: WireSocket) -> std::io::Result<(WireReader, WireWriter)> {
    let writer_stream = socket.stream.try_clone()?;
    let reader = WireReader {
        stream: socket.stream,
        recv_mode: None,
        recv_buf: Vec::new(),
        eof: false,
        stats: Stats::default(),
    };
    let writer = WireWriter {
        stream: writer_stream,
        send_mode: SendMode::Undecided,
        scratch: Vec::new(),
        eof_sent: false,
        stats: Stats::default(),
    };
    Ok((reader, writer))
}
