use crate::serial::{SerialConnection, push_log};
use std::collections::VecDeque;

pub struct MotorState {
    pub running: bool,
    pub rpm: f32,
    pub direction: u8,
    pub reported_rpm: Option<f32>,
    pub diameter_mm: f32,
    pub ms: f32, // meter/second calc'd from RPM and diameter
}

impl Default for MotorState {
    fn default() -> Self {
        // See arduino.ino for the defaults
        let rpm: f32 = 0.1;
        let diameter_mm: f32 = 100.0;
        Self {
            running: true,
            rpm,
            direction: 1,
            reported_rpm: None,
            diameter_mm,
            ms: rpm_to_ms(8.0, 100.0),
        }
    }
}

pub fn rpm_to_ms(rpm: f32, diameter_mm: f32) -> f32 {
    let circumference_m = std::f32::consts::PI * diameter_mm / 1000.0;
    circumference_m * rpm / 60.0
}

pub fn ms_to_rpm(ms: f32, diameter_mm: f32) -> f32 {
    let circumference_m = std::f32::consts::PI * diameter_mm / 1000.0;
    if circumference_m > 0.0 {
        ms * 60.0 / circumference_m
    } else {
        0.0
    }
}

impl MotorState {
    fn set_rpm(&mut self, v: f32) {
        self.rpm = v;
        self.ms = rpm_to_ms(self.rpm, self.diameter_mm);
    }

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
                    self.set_rpm(v);
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
                self.set_rpm(v);
                push_log(log, &format!("RPM set to {:.3}", v));
            }
        } else {
            push_log(log, line);
        }
    }

    pub fn send_start(&self, serial: &mut SerialConnection, log: &mut VecDeque<String>) {
        let cmd = format!("S{:.3}", self.rpm);
        serial.send(&cmd, log);
        serial.send("START", log);
    }

    pub fn send_stop(&self, serial: &mut SerialConnection, log: &mut VecDeque<String>) {
        serial.send("STOP", log);
    }

    pub fn send_rpm(&self, serial: &mut SerialConnection, log: &mut VecDeque<String>) {
        let cmd = format!("S{:.3}", self.rpm);
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
