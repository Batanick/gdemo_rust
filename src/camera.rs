#[allow(dead_code, unused_parens)]

extern crate cgmath;
extern crate winit;

use cgmath::*;
use std::f32::consts::PI;

use window_state;

const MOVE_SPEED: f32 = 10.0;
const ROTATION_SPEED: f32 = 0.005;

pub struct Camera {
    h_angle: f32,
    v_angle: f32,
    pos: Point3<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            h_angle: 0.0,
            v_angle: 0.0,
            pos: Point3::new(0.0, 0.0, -1.0),
        }
    }

    pub fn get_projection(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.pos, self.pos + self.get_direction(), self.get_up())
    }

    pub fn update(&mut self, state: &window_state::WindowState, time: f32) {
        let direction = self.get_direction().normalize() * MOVE_SPEED * time;
        let right = self.get_right().normalize() * MOVE_SPEED * time;

        if state.is_down(winit::VirtualKeyCode::W) {
            self.pos += direction;
        }

        if state.is_down(winit::VirtualKeyCode::S) {
            self.pos += -direction;
        }

        if state.is_down(winit::VirtualKeyCode::A) {
            self.pos += right;
        }

        if state.is_down(winit::VirtualKeyCode::D) {
            self.pos += -right;
        }
    }

    fn get_right(&self) -> Vector3<f32> {
        vec3(f32::sin(self.h_angle - PI / 2.0),
             0.0,
             f32::cos(self.h_angle - PI / 2.0))
    }

    fn get_direction(&self) -> Vector3<f32> {
        vec3(f32::cos(self.v_angle) * f32::sin(self.h_angle),
             f32::sin(self.v_angle),
             f32::cos(self.v_angle) * f32::cos(self.h_angle))
    }

    fn get_up(&self) -> Vector3<f32> {
        self.get_right().cross(self.get_direction())
    }
}
