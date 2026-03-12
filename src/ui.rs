use crate::motor::{MotorState, ms_to_rpm, rpm_to_ms};
use crate::serial::{ConnectionIntent, SerialConnection};
use eframe::egui::{self, Color32, FontId, RichText, Stroke, Vec2};
use std::collections::VecDeque;

const MIN_RPM: f32 = 0.01;
const MAX_RPM: f32 = 100.0;

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = Color32::from_rgb(18, 18, 20);
    ctx.set_style(style);
}

pub fn render_port_bar(
    ui: &mut egui::Ui,
    serial: &mut SerialConnection,
    motor: &mut MotorState,
    log: &mut VecDeque<String>,
) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        ui.label(
            RichText::new("PORT")
                .font(FontId::monospace(11.0))
                .color(Color32::GRAY),
        );
        ui.add_space(5.0);

        egui::ComboBox::from_id_source("port")
            .selected_text(if serial.port_name.is_empty() {
                "Select..."
            } else {
                &serial.port_name
            })
            .width(150.0)
            .show_ui(ui, |ui| {
                for p in serial.available_ports.clone() {
                    ui.selectable_value(&mut serial.port_name, p.clone(), &p);
                }
            });

        ui.add_space(5.0);
        if !serial.connected {
            if ui
                .button(RichText::new("CONNECT").font(FontId::monospace(11.0)))
                .clicked()
                && !serial.port_name.is_empty()
            {
                serial.intent = ConnectionIntent::AutoConnect;
                serial.connect(log);
            }
        } else if ui
            .button(RichText::new("DISCONNECT").font(FontId::monospace(11.0)))
            .clicked()
        {
            serial.intent = ConnectionIntent::ManuallyDisconnected;
            serial.disconnect(log);
            motor.reset();
        }

        ui.add_space(5.0);
        let (dot_color, status_text) = if serial.connected {
            (Color32::from_rgb(80, 220, 80), "ONLINE")
        } else {
            (Color32::from_rgb(180, 60, 60), "OFFLINE")
        };
        ui.colored_label(
            dot_color,
            RichText::new(format!("● {}", status_text)).font(FontId::monospace(11.0)),
        );
    });
}

pub fn render_diameter_input(ui: &mut egui::Ui, motor: &mut MotorState) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(
            RichText::new("DIAMETER  ")
                .font(FontId::monospace(11.0))
                .color(Color32::GRAY),
        );
        ui.add_space(10.0);

        let old_d = motor.diameter_mm;
        let drag = egui::DragValue::new(&mut motor.diameter_mm)
            .clamp_range(1.0..=2000.0)
            .speed(1.0)
            .suffix(" mm");
        if ui.add(drag).changed() && (motor.diameter_mm - old_d).abs() > f32::EPSILON {
            motor.ms = rpm_to_ms(motor.rpm, motor.diameter_mm);
        }
    });
}

pub fn render_rpm_display(ui: &mut egui::Ui, motor: &MotorState) {
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(format!("{:.3}", motor.rpm))
                .font(FontId::monospace(52.0))
                .color(if motor.running {
                    Color32::from_rgb(255, 140, 0)
                } else {
                    Color32::from_rgb(80, 80, 90)
                }),
        );
        ui.label(
            RichText::new("RPM")
                .font(FontId::monospace(13.0))
                .color(Color32::GRAY),
        );
    });
}

pub fn render_rpm_slider(
    ui: &mut egui::Ui,
    motor: &mut MotorState,
    serial: &mut SerialConnection,
    log: &mut VecDeque<String>,
) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(
            RichText::new("SPEED (RPM)")
                .font(FontId::monospace(11.0))
                .color(Color32::GRAY),
        );
        ui.add_space(10.0);

        let old_rpm = motor.rpm;
        let slider = egui::Slider::new(&mut motor.rpm, MIN_RPM..=MAX_RPM).suffix(" RPM");

        if ui.add_sized(Vec2::new(280.0, 20.0), slider).changed()
            && serial.connected
            && (motor.rpm - old_rpm).abs() >= f32::EPSILON
        {
            motor.send_rpm(serial, log);
        }
    });
}

pub fn render_ms_slider(
    ui: &mut egui::Ui,
    motor: &mut MotorState,
    serial: &mut SerialConnection,
    log: &mut VecDeque<String>,
) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(
            RichText::new("SPEED (m/s)")
                .font(FontId::monospace(11.0))
                .color(Color32::GRAY),
        );
        ui.add_space(10.0);

        let old_ms = motor.ms;
        let min_ms = rpm_to_ms(MIN_RPM, motor.diameter_mm);
        let max_ms = rpm_to_ms(MAX_RPM, motor.diameter_mm);
        let slider = egui::Slider::new(&mut motor.ms, min_ms..=max_ms).suffix(" m/s");

        if ui.add_sized(Vec2::new(280.0, 20.0), slider).changed()
            && serial.connected
            && (motor.ms - old_ms).abs() >= f32::EPSILON
        {
            motor.rpm = ms_to_rpm(motor.ms, motor.diameter_mm);
            motor.ms = rpm_to_ms(motor.rpm, motor.diameter_mm);

            if serial.connected {
                motor.send_rpm(serial, log);
            }
        }
    });
}

pub fn render_direction_bar(
    ui: &mut egui::Ui,
    motor: &mut MotorState,
    serial: &mut SerialConnection,
    log: &mut VecDeque<String>,
) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(
            RichText::new("DIR  ")
                .font(FontId::monospace(11.0))
                .color(Color32::GRAY),
        );
        ui.add_space(10.0);

        for (label, value) in [("↻  CW", 0u8), ("↺  CCW", 1u8)] {
            if ui
                .selectable_label(
                    motor.direction == value,
                    RichText::new(label).font(FontId::monospace(12.0)),
                )
                .clicked()
            {
                motor.direction = value;
                if serial.connected {
                    motor.send_direction(serial, log);
                }
            }
            ui.add_space(10.0);
        }

        if ui
            .button(RichText::new("?").font(FontId::monospace(13.0)))
            .clicked()
            && serial.connected
        {
            serial.send("?", log);
        }
    });
}

pub fn render_start_stop(
    ui: &mut egui::Ui,
    motor: &mut MotorState,
    serial: &mut SerialConnection,
    log: &mut VecDeque<String>,
) {
    ui.vertical_centered(|ui| {
        let (btn_text, btn_color) = if motor.running {
            ("■  STOP", Color32::from_rgb(200, 60, 60))
        } else {
            ("▶  START", Color32::from_rgb(60, 180, 60))
        };

        let btn = egui::Button::new(
            RichText::new(btn_text)
                .font(FontId::monospace(15.0))
                .color(Color32::WHITE),
        )
        .fill(btn_color)
        .stroke(Stroke::NONE)
        .min_size(Vec2::new(100.0, 40.0));

        if ui.add(btn).clicked() && serial.connected {
            if motor.running {
                motor.running = false;
                motor.send_stop(serial, log);
            } else {
                motor.running = true;
                motor.send_start(serial, log);
            }
        }
    });
}

pub fn render_log(ui: &mut egui::Ui, log: &VecDeque<String>) {
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label(
            RichText::new("LOG")
                .font(FontId::monospace(10.0))
                .color(Color32::DARK_GRAY),
        );
    });

    egui::ScrollArea::vertical()
        .max_height(100.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(5.0);
            for line in log {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(line)
                            .font(FontId::monospace(10.0))
                            .color(Color32::from_rgb(120, 120, 140)),
                    );
                });
            }
        });
}
