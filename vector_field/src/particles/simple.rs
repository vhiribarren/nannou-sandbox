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

use nannou::{noise::NoiseFn, prelude::*, rand::random_range};

use crate::Radian;

use super::ParticleSystem;

const PARTICLE_SIZE_DEFAULT: f32 = 1.5;
const PARTICLE_MOVE_DELTA: f32 = 2.0;

struct Particle {
    x: f32,
    y: f32,
    color: rgb::Srgb<u8>,
}

pub struct SimpleParticleSystem {
    particles: Vec<Particle>,
    noise: Rc<dyn NoiseFn<[f64; 3]>>,
    container: Rect,
    count: usize,
}

impl SimpleParticleSystem {
    pub fn new(container: Rect, noise: Rc<dyn NoiseFn<[f64; 3]>>, count: usize) -> Self {
        let mut particle_system = Self {
            particles: Vec::with_capacity(count),
            noise,
            count,
            container,
        };
        particle_system.reset();
        particle_system
    }
}

impl ParticleSystem for SimpleParticleSystem {
    fn reset(&mut self) {
        let mut particles = vec![];
        for _ in 0..self.count {
            let x = random_range(self.container.left(), self.container.right());
            let y = random_range(self.container.bottom(), self.container.top());
            particles.push(Particle {
                x,
                y,
                color: Rgb::new(random(), random(), random()),
            });
        }
        self.particles = particles;
    }
    fn update(&mut self, noise_z: f32, frequency: f32, max_angle: Radian) {
        for particle in &mut self.particles {
            let perlin_x = (self.container.right() - particle.x) / self.container.w();
            let perlin_y = (self.container.top() - particle.y) / self.container.h();

            let noise_angle = self.noise.get([
                (perlin_x * frequency) as f64,
                (perlin_y * frequency) as f64,
                noise_z as f64,
            ]) as f32
                * max_angle;
            let gradient = Vec2::new(1., 0.).rotate(noise_angle) * PARTICLE_MOVE_DELTA;
            particle.x += gradient.x;
            particle.y += gradient.y;
        }
    }
    fn draw(&self, draw: &Draw) {
        for particle in &self.particles {
            draw.rect()
                .color(particle.color)
                .w(PARTICLE_SIZE_DEFAULT)
                .h(PARTICLE_SIZE_DEFAULT)
                .x_y(particle.x, particle.y);
        }
    }
}
