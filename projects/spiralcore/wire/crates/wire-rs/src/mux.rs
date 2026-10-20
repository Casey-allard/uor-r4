//! Rust mux gateway for UORADM01: logical streams over one connection, per-stream routes, per-stream
//! compression and 64 KiB window flow control, matching tools/wire_mux.py.
use crate::{compress_chunk, compress_probe, decompress_chunk, level};
use flate2::{Compress, Compression, Decompress};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub const MUX_MAGIC: &[u8; 8] = b"UORADM01";
const INITIAL_WINDOW: u32 = 64 * 1024;
const MAX_FRAME: usize = 65535;
const OPEN: u8 = 1;
const DATA_COMPRESSED: u8 = 2;
const DATA_RAW: u8 = 3;
const FIN: u8 = 4;
const WINDOW: u8 = 5;
const RESET: u8 = 6;

struct MuxWriter {
    stream: TcpStream,
}

impl MuxWriter {
    fn frame(&mut self, stream_id: u16, kind: u8, payload: &[u8]) -> std::io::Result<()> {
        if payload.len() > MAX_FRAME {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "frame too large",
            ));
        }
        let mut header = [0u8; 5];
        header[0..2].copy_from_slice(&stream_id.to_be_bytes());
        header[2] = kind;
        header[3..5].copy_from_slice(&(payload.len() as u16).to_be_bytes());
        self.stream.write_all(&header)?;
        if !payload.is_empty() {
            self.stream.write_all(payload)?;
        }
        Ok(())
    }
}

fn send_window(writer: &Arc<Mutex<MuxWriter>>, stream_id: u16, amount: usize) {
    if let Ok(mut guard) = writer.lock() {
        let _ = guard.frame(stream_id, WINDOW, &(amount as u32).to_be_bytes());
    }
}

struct Credit {
    state: Mutex<u32>,
    ready: Condvar,
    closed: AtomicBool,
}

impl Credit {
    fn new() -> Arc<Self> {
        Arc::new(Credit {
            state: Mutex::new(INITIAL_WINDOW),
            ready: Condvar::new(),
            closed: AtomicBool::new(false),
        })
    }

    fn add(&self, amount: u32) {
        let mut guard = self.state.lock().unwrap();
        *guard = guard.saturating_add(amount);
        self.ready.notify_one();
    }

    fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.ready.notify_all();
    }

    fn take(&self, want: usize) -> Option<usize> {
        let mut guard = self.state.lock().unwrap();
        loop {
            if self.closed.load(Ordering::SeqCst) {
                return None;
            }
            if *guard > 0 {
                let allowed = (*guard as usize).min(want);
                *guard -= allowed as u32;
                return Some(allowed);
            }
            let (next, _) = self
                .ready
                .wait_timeout(guard, Duration::from_millis(50))
                .ok()?;
            guard = next;
        }
    }
}

enum Dispatch {
    Data(Vec<u8>),
    End,
}

struct StreamState {
    dispatch: Sender<Dispatch>,
    credit: Arc<Credit>,
    recv_mode: Option<bool>,
    decompressor: Option<Decompress>,
}

pub struct MuxGateway {
    listener: TcpListener,
    routes: Arc<HashMap<String, String>>,
}

impl MuxGateway {
    pub fn bind(addr: &str, routes: HashMap<String, String>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(MuxGateway {
            listener,
            routes: Arc::new(routes),
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn serve_forever(&self) -> std::io::Result<()> {
        for incoming in self.listener.incoming() {
            match incoming {
                Ok(stream) => {
                    let routes = Arc::clone(&self.routes);
                    thread::spawn(move || {
                        if let Err(error) = handle_connection(stream, routes) {
                            eprintln!("mux connection error: {}", error);
                        }
                    });
                }
                Err(error) => eprintln!("mux accept error: {}", error),
            }
        }
        Ok(())
    }
}

fn resolve_target(
    routes: &HashMap<String, String>,
    requested: &str,
    default_target: Option<&String>,
) -> Option<String> {
    if requested.is_empty() {
        default_target.cloned()
    } else {
        routes.get(requested).cloned()
    }
}

fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<HashMap<String, String>>,
) -> std::io::Result<()> {
    stream.set_nodelay(true).ok();
    let mut magic = [0u8; 8];
    stream.read_exact(&mut magic)?;
    if &magic != MUX_MAGIC {
        return Ok(());
    }
    let mut length = [0u8; 1];
    stream.read_exact(&mut length)?;
    let mut route = vec![0u8; length[0] as usize];
    if !route.is_empty() {
        stream.read_exact(&mut route)?;
    }
    stream.write_all(MUX_MAGIC)?;
    let handshake_route = String::from_utf8_lossy(&route).to_string();
    let default_target = routes.get(&handshake_route).cloned();
    let writer = Arc::new(Mutex::new(MuxWriter {
        stream: stream.try_clone()?,
    }));
    let mut streams: HashMap<u16, StreamState> = HashMap::new();
    loop {
        let mut header = [0u8; 5];
        match stream.read_exact(&mut header) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(error) => return Err(error),
        }
        let stream_id = u16::from_be_bytes([header[0], header[1]]);
        let kind = header[2];
        let size = u16::from_be_bytes([header[3], header[4]]) as usize;
        let mut payload = vec![0u8; size];
        if size > 0 {
            stream.read_exact(&mut payload)?;
        }
        match kind {
            OPEN => {
                let requested = String::from_utf8_lossy(&payload).to_string();
                let target_addr = match resolve_target(&routes, &requested, default_target.as_ref())
                {
                    Some(value) => value,
                    None => {
                        if let Ok(mut guard) = writer.lock() {
                            let _ = guard.frame(stream_id, RESET, &[]);
                        }
                        continue;
                    }
                };
                let mut target = match TcpStream::connect(&target_addr) {
                    Ok(value) => value,
                    Err(_) => {
                        if let Ok(mut guard) = writer.lock() {
                            let _ = guard.frame(stream_id, RESET, &[]);
                        }
                        continue;
                    }
                };
                target.set_nodelay(true).ok();
                let mut source = target.try_clone()?;
                let (dispatch, dispatch_rx) = channel();
                let credit = Credit::new();
                let writer_for_dispatch = Arc::clone(&writer);
                thread::spawn(move || {
                    dispatch_to_target(stream_id, &mut target, dispatch_rx, writer_for_dispatch)
                });
                let writer_for_source = Arc::clone(&writer);
                let credit_for_source = Arc::clone(&credit);
                thread::spawn(move || {
                    stream_to_client(stream_id, &mut source, writer_for_source, credit_for_source)
                });
                streams.insert(
                    stream_id,
                    StreamState {
                        dispatch,
                        credit,
                        recv_mode: None,
                        decompressor: None,
                    },
                );
            }
            DATA_COMPRESSED | DATA_RAW => {
                let state = match streams.get_mut(&stream_id) {
                    Some(value) => value,
                    None => continue,
                };
                if state.recv_mode.is_none() {
                    state.recv_mode = Some(kind == DATA_RAW);
                    if kind == DATA_COMPRESSED {
                        state.decompressor = Some(Decompress::new(true));
                    }
                }
                let data = if state.recv_mode == Some(true) {
                    payload
                } else {
                    match state.decompressor.as_mut() {
                        Some(decompressor) => decompress_chunk(decompressor, &payload),
                        None => continue,
                    }
                };
                if !data.is_empty() {
                    if state.dispatch.send(Dispatch::Data(data)).is_err() {
                        streams.remove(&stream_id);
                    }
                }
            }
            FIN => {
                if let Some(state) = streams.get(&stream_id) {
                    let _ = state.dispatch.send(Dispatch::End);
                }
            }
            WINDOW => {
                if payload.len() == 4 {
                    if let Some(state) = streams.get(&stream_id) {
                        let amount =
                            u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                        state.credit.add(amount);
                    }
                }
            }
            RESET => {
                if let Some(state) = streams.remove(&stream_id) {
                    state.credit.close();
                }
            }
            _ => {}
        }
    }
    for (_, state) in streams.drain() {
        state.credit.close();
        let _ = state.dispatch.send(Dispatch::End);
    }
    Ok(())
}

fn dispatch_to_target(
    stream_id: u16,
    target: &mut TcpStream,
    rx: Receiver<Dispatch>,
    writer: Arc<Mutex<MuxWriter>>,
) {
    while let Ok(message) = rx.recv() {
        match message {
            Dispatch::Data(data) => {
                if target.write_all(&data).is_err() {
                    break;
                }
                send_window(&writer, stream_id, data.len());
            }
            Dispatch::End => break,
        }
    }
    let _ = target.shutdown(std::net::Shutdown::Write);
    let _ = stream_id;
}

fn stream_to_client(
    stream_id: u16,
    source: &mut TcpStream,
    writer: Arc<Mutex<MuxWriter>>,
    credit: Arc<Credit>,
) {
    let mut send_mode: Option<bool> = None;
    let mut compressor: Option<Compress> = None;
    let mut buffer = vec![0u8; 262144];
    loop {
        let read = match source.read(&mut buffer) {
            Ok(0) => break,
            Ok(value) => value,
            Err(_) => break,
        };
        let mut offset = 0usize;
        while offset < read {
            let want = (read - offset).min(60000);
            let allowed = match credit.take(want) {
                Some(value) => value,
                None => return,
            };
            if allowed == 0 {
                return;
            }
            let piece = &buffer[offset..offset + allowed];
            offset += allowed;
            if send_mode.is_none() {
                let probe = compress_probe(piece);
                send_mode = Some(piece.len() >= 4096 && probe.len() >= piece.len());
                if send_mode == Some(false) {
                    compressor = Some(Compress::new(Compression::new(level()), true));
                }
            }
            let (kind, payload) = match send_mode {
                Some(true) => (DATA_RAW, piece.to_vec()),
                _ => {
                    let payload = compress_chunk(compressor.as_mut().unwrap(), piece);
                    (DATA_COMPRESSED, payload)
                }
            };
            if let Ok(mut guard) = writer.lock() {
                if guard.frame(stream_id, kind, &payload).is_err() {
                    return;
                }
            }
        }
    }
    if let Ok(mut guard) = writer.lock() {
        let _ = guard.frame(stream_id, FIN, &[]);
    }
}

// ---------------------------------------------------------------------------------------------
// Native mux client
// ---------------------------------------------------------------------------------------------

struct ClientBuf {
    buf: Vec<u8>,
    eof: bool,
    error: bool,
    recv_mode: Option<bool>,
    decompressor: Option<Decompress>,
}

struct ClientStream {
    state: Mutex<ClientBuf>,
    ready: Condvar,
    credit: Arc<Credit>,
}

struct ClientInner {
    writer: Arc<Mutex<MuxWriter>>,
    streams: Mutex<HashMap<u16, Arc<ClientStream>>>,
    next_id: Mutex<u16>,
}

pub struct MuxClient {
    inner: Arc<ClientInner>,
}

impl MuxClient {
    pub fn connect(addr: &str, route: &str) -> std::io::Result<Self> {
        let mut stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true).ok();
        let encoded = route.as_bytes();
        let length = encoded.len().min(255) as u8;
        let mut handshake = Vec::with_capacity(8 + 1 + length as usize);
        handshake.extend_from_slice(MUX_MAGIC);
        handshake.push(length);
        handshake.extend_from_slice(&encoded[..length as usize]);
        stream.write_all(&handshake)?;
        let mut reply = [0u8; 8];
        stream.read_exact(&mut reply)?;
        if &reply != MUX_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "mux handshake failed",
            ));
        }
        let writer = Arc::new(Mutex::new(MuxWriter {
            stream: stream.try_clone()?,
        }));
        let inner = Arc::new(ClientInner {
            writer,
            streams: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        });
        let reader_inner = Arc::clone(&inner);
        let reader_stream = stream.try_clone()?;
        thread::spawn(move || client_reader(reader_inner, reader_stream));
        Ok(MuxClient { inner })
    }

    pub fn open_stream(&self, route: &str) -> std::io::Result<MuxStream> {
        let stream_id = {
            let mut next = self.inner.next_id.lock().unwrap();
            let id = *next;
            *next = next.wrapping_add(1);
            id
        };
        let shared = Arc::new(ClientStream {
            state: Mutex::new(ClientBuf {
                buf: Vec::new(),
                eof: false,
                error: false,
                recv_mode: None,
                decompressor: None,
            }),
            ready: Condvar::new(),
            credit: Credit::new(),
        });
        self.inner
            .streams
            .lock()
            .unwrap()
            .insert(stream_id, Arc::clone(&shared));
        {
            let mut writer = self.inner.writer.lock().unwrap();
            writer.frame(stream_id, OPEN, route.as_bytes())?;
        }
        Ok(MuxStream {
            stream_id,
            inner: Arc::clone(&self.inner),
            shared,
            send_mode: None,
            compressor: None,
            fin_sent: false,
        })
    }
}

pub struct MuxStream {
    stream_id: u16,
    inner: Arc<ClientInner>,
    shared: Arc<ClientStream>,
    send_mode: Option<bool>,
    compressor: Option<Compress>,
    fin_sent: bool,
}

impl MuxStream {
    pub fn send_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        if self.fin_sent {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "send after shutdown",
            ));
        }
        let mut offset = 0usize;
        while offset < data.len() {
            let want = (data.len() - offset).min(60000);
            let allowed = self
                .shared
                .credit
                .take(want)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "closed"))?;
            if allowed == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "closed",
                ));
            }
            let piece = &data[offset..offset + allowed];
            offset += allowed;
            if self.send_mode.is_none() {
                let probe = compress_probe(piece);
                self.send_mode = Some(piece.len() >= 4096 && probe.len() >= piece.len());
                if self.send_mode == Some(false) {
                    self.compressor = Some(Compress::new(Compression::new(level()), true));
                }
            }
            let (kind, payload) = match self.send_mode {
                Some(true) => (DATA_RAW, piece.to_vec()),
                _ => {
                    let payload = compress_chunk(self.compressor.as_mut().unwrap(), piece);
                    (DATA_COMPRESSED, payload)
                }
            };
            let mut writer = self.inner.writer.lock().unwrap();
            writer.frame(self.stream_id, kind, &payload)?;
        }
        Ok(())
    }

    pub fn shutdown(&mut self) -> std::io::Result<()> {
        if !self.fin_sent {
            self.fin_sent = true;
            let mut writer = self.inner.writer.lock().unwrap();
            writer.frame(self.stream_id, FIN, &[])?;
        }
        Ok(())
    }

    pub fn recv(&mut self, max: usize) -> std::io::Result<Vec<u8>> {
        let mut state = self.shared.state.lock().unwrap();
        while state.buf.is_empty() && !state.eof && !state.error {
            state = self.shared.ready.wait(state).unwrap();
        }
        if state.error {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "stream reset",
            ));
        }
        let take = state.buf.len().min(max);
        let data: Vec<u8> = state.buf.drain(..take).collect();
        drop(state);
        if take > 0 {
            let mut writer = self.inner.writer.lock().unwrap();
            writer.frame(self.stream_id, WINDOW, &(take as u32).to_be_bytes())?;
        }
        Ok(data)
    }

    pub fn close(mut self) {
        let _ = self.shutdown();
        self.inner.streams.lock().unwrap().remove(&self.stream_id);
    }
}

fn client_reader(inner: Arc<ClientInner>, mut stream: TcpStream) {
    loop {
        let mut header = [0u8; 5];
        match stream.read_exact(&mut header) {
            Ok(()) => {}
            Err(_) => break,
        }
        let stream_id = u16::from_be_bytes([header[0], header[1]]);
        let kind = header[2];
        let size = u16::from_be_bytes([header[3], header[4]]) as usize;
        let mut payload = vec![0u8; size];
        if size > 0 && stream.read_exact(&mut payload).is_err() {
            break;
        }
        let shared = inner.streams.lock().unwrap().get(&stream_id).cloned();
        let shared = match shared {
            Some(value) => value,
            None => continue,
        };
        match kind {
            DATA_COMPRESSED | DATA_RAW => {
                let mut state = shared.state.lock().unwrap();
                if state.recv_mode.is_none() {
                    state.recv_mode = Some(kind == DATA_RAW);
                    if kind == DATA_COMPRESSED {
                        state.decompressor = Some(Decompress::new(true));
                    }
                }
                let data = if state.recv_mode == Some(true) {
                    payload
                } else {
                    match state.decompressor.as_mut() {
                        Some(decompressor) => decompress_chunk(decompressor, &payload),
                        None => continue,
                    }
                };
                state.buf.extend_from_slice(&data);
                shared.ready.notify_all();
            }
            FIN => {
                let mut state = shared.state.lock().unwrap();
                state.eof = true;
                shared.ready.notify_all();
            }
            WINDOW => {
                if payload.len() == 4 {
                    let amount =
                        u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    shared.credit.add(amount);
                }
            }
            RESET => {
                let mut state = shared.state.lock().unwrap();
                state.error = true;
                shared.ready.notify_all();
            }
            _ => {}
        }
    }
    let streams: Vec<Arc<ClientStream>> = inner.streams.lock().unwrap().values().cloned().collect();
    for shared in streams {
        let mut state = shared.state.lock().unwrap();
        state.eof = true;
        shared.ready.notify_all();
    }
}
