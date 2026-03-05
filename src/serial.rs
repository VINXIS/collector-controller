use serialport::SerialPort;
use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum ConnectionIntent {
    AutoConnect,
    ManuallyDisconnected,
}

pub struct SerialConnection {
    pub port_name: String,
    pub available_ports: Vec<String>,
    pub intent: ConnectionIntent,
    pub connected: bool,
    port: Option<Arc<Mutex<Box<dyn SerialPort>>>>,
    buf: String,
    pub last_port_scan: Instant,
    pub last_ping: Instant,
    pub last_connection_check: Instant,
}

impl SerialConnection {
    pub fn new() -> Self {
        let ports = Self::scan_ports();
        Self {
            port_name: String::new(),
            available_ports: ports,
            intent: ConnectionIntent::AutoConnect,
            connected: false,
            port: None,
            buf: String::new(),
            last_port_scan: Instant::now(),
            last_ping: Instant::now(),
            last_connection_check: Instant::now(),
        }
    }

    pub fn scan_ports() -> Vec<String> {
        serialport::available_ports()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.port_name)
            .collect()
    }

    pub fn connect(&mut self, log: &mut VecDeque<String>) {
        match serialport::new(&self.port_name, 115200)
            .timeout(Duration::from_millis(50))
            .open()
        {
            Ok(p) => {
                self.port = Some(Arc::new(Mutex::new(p)));
                self.connected = true;
                push_log(log, &format!("Connected to {}", self.port_name));
                std::thread::sleep(Duration::from_millis(1800));
                self.send("?", log);
                self.last_ping = Instant::now();
            }
            Err(e) => {
                push_log(
                    log,
                    &format!("Failed to connect to {}: {e}", self.port_name),
                );
            }
        }
    }

    pub fn disconnect(&mut self, log: &mut VecDeque<String>) {
        self.port = None;
        self.connected = false;
        push_log(log, "Disconnected.");
    }

    pub fn send(&mut self, cmd: &str, log: &mut VecDeque<String>) {
        let failed = if let Some(port) = &self.port {
            if let Ok(mut p) = port.lock() {
                let msg = format!("{}\n", cmd);
                p.write_all(msg.as_bytes()).is_err()
            } else {
                false
            }
        } else {
            false
        };

        if failed {
            self.mark_disconnected(log);
        }
    }

    pub fn mark_disconnected(&mut self, log: &mut VecDeque<String>) {
        self.connected = false;
        self.port = None;
        self.intent = ConnectionIntent::AutoConnect;
        push_log(log, "Connection lost.");
    }

    pub fn is_alive(&self) -> bool {
        self.port
            .as_ref()
            .and_then(|p| p.lock().ok())
            .map(|p| p.bytes_to_read().is_ok())
            .unwrap_or(false)
    }

    /// Returns lines received since last call.
    pub fn poll(&mut self) -> Vec<String> {
        let Some(port) = &self.port else {
            return vec![];
        };
        let mut raw = [0u8; 256];
        let n = port
            .lock()
            .ok()
            .map(|mut p| p.read(&mut raw).unwrap_or(0))
            .unwrap_or(0);
        if n == 0 {
            return vec![];
        }
        self.buf.push_str(&String::from_utf8_lossy(&raw[..n]));

        let mut lines = Vec::new();
        while let Some(pos) = self.buf.find('\n') {
            let line = self.buf[..pos].trim().to_string();
            self.buf = self.buf[pos + 1..].to_string();
            if !line.is_empty() {
                lines.push(line);
            }
        }
        lines
    }
}

pub fn push_log(log: &mut VecDeque<String>, msg: &str) {
    log.push_back(msg.to_string());
    if log.len() > 60 {
        log.pop_front();
    }
}
