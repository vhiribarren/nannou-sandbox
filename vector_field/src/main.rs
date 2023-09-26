/*
MIT License

Copyright (c) 2023 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

use std::rc::Rc;

use nannou::{
    color::IntoLinSrgba,
    draw::Renderer,
    noise::{NoiseFn, Perlin},
    prelude::*,
};
use nannou_egui::{
    egui::{self},
    Egui,
};
use vector_field::{
    particles::{simple::SimpleParticleSystem, ParticleSystem},
    Radian,
};

const ARROW_COLOR: rgb::Srgb<u8> = BLACK;
const BACKGROUND_COLOR: rgb::Srgb<u8> = CORNFLOWERBLUE;
const SPEED_DEFAULT: f32 = 0.1;
const STEP_DEFAULT: usize = 50;
const MAX_ANGLE_DEFAULT: Radian = 2.0 * PI;
const RUNNING_DEFAULT: bool = false;
const SHOW_ARROWS_DEFAULT: bool = true;
const SHOW_VALUES_DEFAULT: bool = false;
const FREQUENCY_DEFAULT: f32 = 1.0;

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    egui: Egui,
    show_arrows: bool,
    show_values: bool,
    running: bool,
    reference_time: f32,
    speed: f32,
    step_sample: usize,
    max_angle: Radian,
    noise: Rc<dyn NoiseFn<[f64; 3]>>,
    frequency: f32,
    particle_system: Box<dyn ParticleSystem>,
    particle_texture: wgpu::Texture,
    enable_particles: bool,
    renderer: Renderer,
    angle_color: AngleColor,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Debug)]
enum AngleColor {
    Gray,
    HSV,
}

fn model(app: &App) -> Model {
    fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
        model.egui.handle_raw_event(event);
    }
    let window = {
        let window_id = app
            .new_window()
            .view(view)
            .raw_event(raw_window_event)
            .build()
            .unwrap();
        app.window(window_id).unwrap()
    };
    let egui = Egui::from_window(&window);
    let noise = Rc::new(Perlin::new());
    let particle_system = Box::new(SimpleParticleSystem::new(window.rect(), noise.clone()));
    let particle_texture = wgpu::TextureBuilder::new()
        .size([window.rect().w() as u32, window.rect().h() as u32])
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        .sample_count(1) //.sample_count(window.msaa_samples())
        .format(wgpu::TextureFormat::Rgba16Float)
        .build(window.device());
    let renderer = {
        let descriptor = particle_texture.descriptor();
        nannou::draw::RendererBuilder::new()
            .build_from_texture_descriptor(window.device(), descriptor)
    };
    Model {
        egui,
        running: RUNNING_DEFAULT,
        show_arrows: SHOW_ARROWS_DEFAULT,
        show_values: SHOW_VALUES_DEFAULT,
        reference_time: 0_f32,
        speed: SPEED_DEFAULT,
        step_sample: STEP_DEFAULT,
        max_angle: MAX_ANGLE_DEFAULT,
        noise: noise.clone(),
        frequency: FREQUENCY_DEFAULT,
        particle_system,
        particle_texture,
        renderer,
        enable_particles: false,
        angle_color: AngleColor::Gray,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let noise_z = noise_z(app, model) as f32;

    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.vertical(|ui| {
            ui.heading("Noise control");
            ui.add(egui::Slider::new(&mut model.step_sample, 1..=100).text("Steps"));
            ui.add(
                egui::Slider::new(&mut model.max_angle, 0.0..=2.0 * PI)
                    .text("Max angle")
                    .suffix("rad"),
            );
            ui.add(
                egui::Slider::new(&mut model.frequency, 0.1..=100.0)
                    .text("Frequency")
                    .logarithmic(true),
            );
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Angle Color Selection")
                    .selected_text(format!("{:?}", model.angle_color))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut model.angle_color, AngleColor::Gray, "Gray");
                        ui.selectable_value(&mut model.angle_color, AngleColor::HSV, "Hue");
                    });
                ui.checkbox(&mut model.show_values, "Show Values");
                ui.checkbox(&mut model.show_arrows, "Show Arrows");
            });
            ui.separator();
            ui.heading("Update vector field");
            ui.add(
                egui::Slider::new(&mut model.speed, 0.0..=100.0)
                    .text("Speed")
                    .logarithmic(true),
            );
            if ui
                .button(if model.running { "Pause" } else { "Run" })
                .clicked()
            {
                model.reference_time = app.time * model.speed - model.reference_time;
                model.running = !model.running;
            }
            ui.separator();
            ui.heading("Particles");
            ui.horizontal(|ui| {
                if ui.button("Reset particles").clicked() {
                    model.particle_system.reset();
                    model.particle_texture = wgpu::TextureBuilder::new()
                        .size([
                            app.main_window().rect().w() as u32,
                            app.main_window().rect().h() as u32,
                        ])
                        .usage(
                            wgpu::TextureUsages::RENDER_ATTACHMENT
                                | wgpu::TextureUsages::TEXTURE_BINDING,
                        )
                        .sample_count(1) //.sample_count(window.msaa_samples())
                        .format(wgpu::TextureFormat::Rgba16Float)
                        .build(app.main_window().device());
                }
                ui.checkbox(&mut model.enable_particles, "Enable particles");
            });
            model.particle_system.config_gui(ui);
        });
    });

    if model.enable_particles {
        let draw = app.draw();
        let window = app.main_window();
        let device = window.device();
        let ce_desc = wgpu::CommandEncoderDescriptor {
            label: Some("texture renderer"),
        };
        let mut encoder = device.create_command_encoder(&ce_desc);

        model
            .particle_system
            .update(noise_z, model.frequency, model.max_angle);
        model.particle_system.draw(&draw);
        model
            .renderer
            .render_to_texture(device, &mut encoder, &draw, &model.particle_texture);
        window.queue().submit(Some(encoder.finish()));
    }
}

fn noise_z(app: &App, model: &Model) -> f64 {
    if model.running {
        (app.time * model.speed - model.reference_time) as f64
    } else {
        model.reference_time as f64
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let step = model.step_sample;
    let arrow_width = (step - 2) as f32;
    let stroke_weight = 2.;
    let max_angle = model.max_angle;
    let win = app.window_rect();
    let perlin_z = noise_z(app, model);

    draw.background().color(BACKGROUND_COLOR);

    for canvas_x in (win.left() as i32..win.right() as i32).step_by(step) {
        for canvas_y in (win.bottom() as i32..win.top() as i32).step_by(step) {
            let perlin_x = (win.right() - canvas_x as f32) / win.w();
            let perlin_y = (win.top() - canvas_y as f32) / win.h();
            let noise_angle = model.noise.get([
                (perlin_x * model.frequency) as f64,
                (perlin_y * model.frequency) as f64,
                perlin_z,
            ]) as f32
                * max_angle;
            let gradient = Vec2::new(1., 0.).rotate(noise_angle as f32) * arrow_width;
            let canvas_point = Vec2::new(canvas_x as f32, canvas_y as f32);
            let offset = Vec2::new(gradient.x / 2., gradient.y / 2.);
            if model.show_values {
                let color = match model.angle_color {
                    AngleColor::Gray => {
                        let gray = (noise_angle.cos() + 1.0) / 2.0;
                        Rgb::new(gray, gray, gray).into_lin_srgba()
                    }
                    AngleColor::HSV => {
                        Hsv::new(noise_angle * 360.0 / (2. * PI), 1.0, 1.0).into_lin_srgba()
                    }
                };
                draw.rect().color(color).w(step as f32).h(step as f32).x_y(
                    canvas_x as f32 + step as f32 / 2.0,
                    canvas_y as f32 + step as f32 / 2.0,
                );
            }

            if model.show_arrows {
                draw.arrow()
                    .start(canvas_point - offset)
                    .end(canvas_point + offset)
                    .stroke_weight(stroke_weight)
                    .color(ARROW_COLOR);
            }
        }
    }
    draw.texture(&model.particle_texture);
    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
