extern crate winit;

use std::collections::HashMap;

#[derive(Default)]
pub struct WindowState {
    states: HashMap<winit::VirtualKeyCode, bool>,

    height: f32,
    width: f32,

    mouse_x: f32,
    mouse_y: f32,
}

impl WindowState {
    pub fn new(window: &winit::Window) -> Self {
        let size = window.get_inner_size_points();
        let size = size.unwrap_or_default();

        WindowState {
            states: HashMap::new(),

            width: size.0 as f32,
            height: size.1 as f32,

            mouse_x: (size.0 / 2) as f32,
            mouse_y: (size.1 / 2) as f32,
        }
    }

    pub fn switch(&mut self, code: winit::VirtualKeyCode, down: bool) {
        self.states.insert(code, down);
    }

    pub fn is_down(&self, key: winit::VirtualKeyCode) -> bool {
        self.states.get(&key).unwrap_or(&false).clone()
    }

    pub fn get_mouse_pos(&self) -> (f32, f32) {
        (self.mouse_x, self.mouse_y).clone()
    }

    pub fn get_window_size(&self) -> (f32, f32) {
        (self.height, self.height).clone()
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.height = height as f32;
        self.width = width as f32;
    }

    pub fn update_mouse(&mut self, mouse_x: i32, mouse_y: i32) {
        self.mouse_x = mouse_x as f32;
        self.mouse_x = mouse_y as f32;
    }

    pub fn get_aspect(&self) -> f32 {
        self.width / self.height
    }
}
