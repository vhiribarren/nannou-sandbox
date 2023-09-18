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

use nannou::{
    noise::{NoiseFn, Perlin},
    prelude::*,
};
use nannou_egui::{egui, Egui};

const ARROW_COLOR: rgb::Srgb<u8> = BLACK;
const BACKGROUND_COLOR: rgb::Srgb<u8> = CORNFLOWERBLUE;
const SPEED_DEFAULT: f64 = 0.1;
const STEP_DEFAULT: usize = 50;
const MAX_ANGLE_DEFAULT: f64 = 2.0 * PI_F64;
const RUNNING_DEFAULT: bool = true;

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

struct Model {
    egui: Egui,
    running: bool,
    reference_time: f32,
    speed: f64,
    step: usize,
    max_angle: f64,
    noise: Box<dyn NoiseFn<[f64; 3]>>,
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
    Model {
        egui,
        running: RUNNING_DEFAULT,
        reference_time: 0_f32,
        speed: SPEED_DEFAULT,
        step: STEP_DEFAULT,
        max_angle: MAX_ANGLE_DEFAULT,
        noise: Box::new(Perlin::new()),
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Settings").show(&ctx, |ui| {
        ui.add(
            egui::Slider::new(&mut model.speed, 0.0..=100.0)
                .text("Speed")
                .logarithmic(true),
        );
        ui.add(egui::Slider::new(&mut model.step, 1..=100).text("Steps"));
        ui.add(
            egui::Slider::new(&mut model.max_angle, 0.0..=2.0 * PI_F64)
                .text("Max angle")
                .suffix("rad"),
        );
        if ui.button("Run/Pause").clicked() {
            model.reference_time = app.time * model.speed as f32 - model.reference_time as f32;
            model.running = !model.running;
        }
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let step = model.step;
    let arrow_width = (step - 2) as f32;
    let stroke_weight = 2.;
    let time_factor = model.speed;
    let max_angle = model.max_angle;
    let perlin_z = if model.running {
        app.time as f64 * time_factor - model.reference_time as f64
    } else {
        model.reference_time as f64
    };

    draw.background().color(BACKGROUND_COLOR);

    let win = app.window_rect();
    for canvas_x in (win.left() as i32..win.right() as i32).step_by(step) {
        for canvas_y in (win.bottom() as i32..win.top() as i32).step_by(step) {
            let perlin_x = win.right() as f64 - canvas_x as f64 / win.w() as f64;
            let perlin_y = win.top() as f64 - canvas_y as f64 / win.h() as f64;
            let noise_angle = model.noise.get([perlin_x, perlin_y, perlin_z]) * max_angle;
            let gradient = Vec2::new(1., 0.).rotate(noise_angle as f32) * arrow_width;
            let canvas_point = Vec2::new(canvas_x as f32, canvas_y as f32);
            let offset = Vec2::new(gradient.x / 2., gradient.y / 2.);
            draw.arrow()
                .start(canvas_point - offset)
                .end(canvas_point + offset)
                .stroke_weight(stroke_weight)
                .color(ARROW_COLOR);
        }
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
