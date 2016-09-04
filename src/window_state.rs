extern crate winit;

use std::collections::HashMap;

#[derive(Default)]
pub struct WindowState {
    states: HashMap<winit::VirtualKeyCode, bool>,

    height: u32,
    width: u32,

    mouse_x: i32,
    mouse_y: i32,
}

impl WindowState {
    pub fn new(window: &winit::Window) -> Self {
        let size = window.get_inner_size_points();
        let size = size.unwrap_or_default();

        WindowState {
            states: HashMap::new(),

            width: size.0,
            height: size.1,

            mouse_x: (size.0 / 2) as i32,
            mouse_y: (size.1 / 2) as i32,
        }
    }

    pub fn switch(&mut self, code: winit::VirtualKeyCode, down: bool) {
        self.states.insert(code, down);
    }

    pub fn is_down(&self, key: winit::VirtualKeyCode) -> bool {
        self.states.get(&key).unwrap_or(&false).clone()
    }

    pub fn get_mouse_pos(&self) -> (i32, i32) {
        (self.mouse_x, self.mouse_y).clone()
    }

    pub fn get_window_size(&self) -> (u32, u32) {
        (self.height, self.height).clone()
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.height = height;
        self.width = width;
    }

    pub fn update_mouse(&mut self, mouse_x: i32, mouse_y: i32) {
        self.mouse_x = mouse_x;
        self.mouse_x = mouse_y;
    }
}
