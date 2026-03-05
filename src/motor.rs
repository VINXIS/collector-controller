use crate::serial::{SerialConnection, push_log};
use std::collections::VecDeque;

pub struct MotorState {
    pub running: bool,
    pub rpm: f32,
    pub direction: u8,
    pub reported_rpm: Option<f32>,
}

impl Default for MotorState {
    fn default() -> Self {
        // See arduino.ino for the defaults
        Self {
            running: true,
            rpm: 8.0,
            direction: 1,
            reported_rpm: None,
        }
    }
}

impl MotorState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn handle_line(&mut self, line: &str, log: &mut VecDeque<String>) {
        if line.starts_with("RPM:") {
            if let Ok(v) = line[4..].parse::<f32>() {
                self.reported_rpm = Some(v);
            }
        } else if line.starts_with("STATE:") {
            self.running = line.contains("RUNNING");
            if let Some(rpm_part) = line.split("RPM:").nth(1) {
                if let Ok(v) = rpm_part
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .parse::<f32>()
                {
                    self.rpm = v;
                }
            }
            if let Some(dir_part) = line.split("DIR:").nth(1) {
                self.direction = dir_part.trim().parse::<u8>().unwrap_or(1);
            }
            push_log(log, line);
        } else if line == "RUNNING" {
            self.running = true;
        } else if line == "STOPPED" {
            self.running = false;
            self.reported_rpm = None;
        } else if line.starts_with("RPM_SET:") {
            if let Ok(v) = line[8..].parse::<f32>() {
                self.rpm = v;
                push_log(log, &format!("RPM set to {:.0}", v));
            }
        } else {
            push_log(log, line);
        }
    }

    pub fn send_start(&self, serial: &mut SerialConnection, log: &mut VecDeque<String>) {
        let cmd = format!("S{:.0}", self.rpm);
        serial.send(&cmd, log);
        serial.send("START", log);
    }

    pub fn send_stop(&self, serial: &mut SerialConnection, log: &mut VecDeque<String>) {
        serial.send("STOP", log);
    }

    pub fn send_rpm(&self, serial: &mut SerialConnection, log: &mut VecDeque<String>) {
        let cmd = format!("S{:.0}", self.rpm);
        serial.send(&cmd, log);
    }

    pub fn send_direction(&self, serial: &mut SerialConnection, log: &mut VecDeque<String>) {
        let cmd = format!("D{}", self.direction);
        serial.send(&cmd, log);
    }

    pub fn display_rpm(&self) -> f32 {
        self.reported_rpm.unwrap_or(self.rpm)
    }
}
