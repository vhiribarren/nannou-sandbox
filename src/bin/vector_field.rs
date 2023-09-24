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
    noise::{NoiseFn, Perlin},
    prelude::*, draw::Renderer,
};
use nannou_egui::{egui, Egui};

type Radian = f32;

const ARROW_COLOR: rgb::Srgb<u8> = BLACK;
const BACKGROUND_COLOR: rgb::Srgb<u8> = CORNFLOWERBLUE;
const SPEED_DEFAULT: f32 = 0.1;
const STEP_DEFAULT: usize = 50;
const MAX_ANGLE_DEFAULT: Radian = 2.0 * PI;
const RUNNING_DEFAULT: bool = true;
const SHOW_ARROWS_DEFAULT: bool = true;
const SHOW_VALUES_DEFAULT: bool = false;
const FREQUENCY_DEFAULT: f32 = 1.0;
const PARTICLE_COUNT_DEFAULT: usize = 5_000;
const PARTICLE_COLOR_DEFAULT: rgb::Srgb<u8> = RED;
const PARTICLE_SIZE_DEFAULT: f32 = 1.5;
const PARTICLE_MOVE_DELTA: f32 = 1.0;

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Particle {
    pub x: f32,
    pub y: f32,
    pub color: rgb::Srgb<u8>,
}

struct ParticleSystem {
    particles: Vec<Particle>,
    noise: Rc<dyn NoiseFn<[f64; 3]>>,
    container: Rect,
}

impl ParticleSystem {
    fn new(container: Rect, noise: Rc<dyn NoiseFn<[f64; 3]>>, count: usize) -> Self {
        let mut particles = vec![];
        for _ in 0..count {
            let x = random_range(container.left(), container.right());
            let y = random_range(container.bottom(), container.top());
            particles.push(Particle {
                x,
                y,
                color: PARTICLE_COLOR_DEFAULT,
            });
        }
        Self {
            particles,
            noise,
            container,
        }
    }
    fn update(&mut self, draw: &Draw, app: &App, renderer: &mut Renderer, particle_texture: &wgpu::Texture, noise_z: f32, frequency: f32, max_angle: Radian) {
        draw.reset();
        for particle in &mut self.particles {
            let perlin_x =
                (self.container.right() - particle.x) / self.container.w();
            let perlin_y =
                (self.container.top() - particle.y) / self.container.h();

            let noise_angle = self
                .noise
                .get([(perlin_x * frequency) as f64, (perlin_y * frequency) as f64, noise_z as f64]) as f32
                * max_angle;
            let gradient = Vec2::new(1., 0.).rotate(noise_angle) * PARTICLE_MOVE_DELTA;
            particle.x += gradient.x;
            particle.y += gradient.y;
        }

        for particle in &self.particles {
            draw.rect()
                .color(particle.color)
                .w(PARTICLE_SIZE_DEFAULT)
                .h(PARTICLE_SIZE_DEFAULT)
                .x_y(particle.x, particle.y);
        }

        let window = app.main_window();
        let device = window.device();
        let ce_desc = wgpu::CommandEncoderDescriptor {
            label: Some("texture renderer"),
        };
        let mut encoder = device.create_command_encoder(&ce_desc);
        renderer
            .render_to_texture(device, &mut encoder, &draw, particle_texture);
        window.queue().submit(Some(encoder.finish()));



    }
    fn draw(&self, app: &App, model: &Model, frame: &Frame) {

    }
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
    particle_system: ParticleSystem,
    particle_texture: wgpu::Texture,
    renderer: Renderer,
    draw: Draw,
}

fn model(app: &App) -> Model {
    fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
        model.egui.handle_raw_event(event);
    }
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let egui = Egui::from_window(&window);
    let noise = Rc::new(Perlin::new());
    let particle_system = ParticleSystem::new(window.rect(), noise.clone(), PARTICLE_COUNT_DEFAULT);
    let particle_texture = wgpu::TextureBuilder::new()
        .size([window.rect().w() as u32, window.rect().h() as u32])
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        .sample_count(1)//.sample_count(window.msaa_samples())
        .format(wgpu::TextureFormat::Rgba16Float)
        .build(window.device());
    let descriptor = particle_texture.descriptor();
    let renderer =
        nannou::draw::RendererBuilder::new().build_from_texture_descriptor(window.device(), descriptor);
    let draw = nannou::Draw::new();
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
        draw,
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let noise_z = noise_z(app, model) as f32;
    model
        .particle_system
        .update(&model.draw, app, &mut model.renderer, &model.particle_texture, noise_z, model.frequency, model.max_angle);
    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.vertical(|ui| {
            ui.add(
                egui::Slider::new(&mut model.speed, 0.0..=100.0)
                    .text("Speed")
                    .logarithmic(true),
            );
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
            ui.checkbox(&mut model.show_arrows, "Show Arrows");
            ui.checkbox(&mut model.show_values, "Show Values");
            if ui
                .button(if model.running { "Pause" } else { "Run" })
                .clicked()
            {
                model.reference_time = app.time * model.speed - model.reference_time;
                model.running = !model.running;
            }
        });
    });
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
    let perlin_z = noise_z(app, model);

    draw.background().color(BACKGROUND_COLOR);

    draw.texture(&model.particle_texture);

    let win = app.window_rect();
    for canvas_x in (win.left() as i32..win.right() as i32).step_by(step) {
        for canvas_y in (win.bottom() as i32..win.top() as i32).step_by(step) {
            let perlin_x = (win.right() - canvas_x as f32) / win.w();
            let perlin_y = (win.top() - canvas_y as f32) / win.h();
            let noise_angle = model.noise.get([
                (perlin_x * model.frequency) as f64,
                (perlin_y * model.frequency) as f64,
                perlin_z,
            ]) as f32* max_angle;
            let gradient = Vec2::new(1., 0.).rotate(noise_angle as f32) * arrow_width;
            let canvas_point = Vec2::new(canvas_x as f32, canvas_y as f32);
            let offset = Vec2::new(gradient.x / 2., gradient.y / 2.);
            if model.show_values {
                draw.rect()
                    .color(Rgb::new(noise_angle, noise_angle, noise_angle))
                    .w(step as f32)
                    .h(step as f32)
                    .x_y(
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


    draw.to_frame(app, &frame).unwrap();
    //model.particle_system.draw(app, model, &frame);

    model.egui.draw_to_frame(&frame).unwrap();
}
