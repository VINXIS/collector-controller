use crate::motor::MotorState;
use crate::serial::{ConnectionIntent, SerialConnection};
use crate::ui;
use eframe::egui;
use std::collections::VecDeque;
use std::time::Duration;

pub struct App {
    pub serial: SerialConnection,
    pub motor: MotorState,
    pub log: VecDeque<String>,
}

impl App {
    pub fn new() -> Self {
        let mut log = VecDeque::new();
        log.push_back("Scanning for devices...".to_string());
        Self {
            serial: SerialConnection::new(),
            motor: MotorState::default(),
            log,
        }
    }

    fn tick_port_scan(&mut self) {
        if self.serial.last_port_scan.elapsed() <= std::time::Duration::from_secs(2) {
            return;
        }
        let new_ports = SerialConnection::scan_ports();

        if !self.serial.connected && self.serial.intent == ConnectionIntent::AutoConnect {
            let candidate = new_ports
                .iter()
                .find(|p| !self.serial.available_ports.contains(p))
                .or_else(|| new_ports.first())
                .cloned();

            if let Some(port) = candidate {
                if !self.serial.connected {
                    self.serial.port_name = port;
                    self.serial.connect(&mut self.log);
                }
            }
        }
        self.serial.available_ports = new_ports;
        self.serial.last_port_scan = std::time::Instant::now();
    }

    fn tick_ping(&mut self) {
        if self.serial.connected && self.serial.last_ping.elapsed() > Duration::from_secs(2) {
            self.serial.send("?", &mut self.log);
            self.serial.last_ping = std::time::Instant::now();
        }
    }

    fn tick_health_check(&mut self) {
        if self.serial.connected
            && self.serial.last_connection_check.elapsed() > Duration::from_secs(3)
        {
            if !self.serial.is_alive() {
                self.serial.mark_disconnected(&mut self.log);
                self.motor.reset();
            }
            self.serial.last_connection_check = std::time::Instant::now();
        }
    }

    fn tick_serial(&mut self) {
        let lines = self.serial.poll();
        for line in lines {
            self.motor.handle_line(&line, &mut self.log);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Background ticks and update
        self.tick_serial();
        self.tick_port_scan();
        self.tick_ping();
        self.tick_health_check();
        ctx.request_repaint_after(Duration::from_millis(150));

        ui::apply_theme(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.add_space(15.0);
                ui.label(
                    eframe::egui::RichText::new("COLLECTOR CONTROLLER")
                        .font(eframe::egui::FontId::monospace(18.0))
                        .color(eframe::egui::Color32::from_rgb(255, 140, 0)),
                );
            });

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            ui::render_port_bar(ui, &mut self.serial, &mut self.motor, &mut self.log);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui::render_rpm_display(ui, &self.motor);

            ui.add_space(10.0);
            ui::render_speed_slider(ui, &mut self.motor, &mut self.serial, &mut self.log);

            ui.add_space(10.0);
            ui::render_direction_bar(ui, &mut self.motor, &mut self.serial, &mut self.log);

            ui.add_space(10.0);
            ui::render_start_stop(ui, &mut self.motor, &mut self.serial, &mut self.log);

            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);

            ui::render_log(ui, &self.log);
        });
    }
}
