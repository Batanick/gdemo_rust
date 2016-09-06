#![allow(dead_code)]
#[macro_use]
extern crate vulkano;
extern crate winit;
extern crate vulkano_win;
extern crate cgmath;
extern crate time;

mod renderer;
mod window_state;
mod camera;
mod fps_counter;

fn main() {
    let mut renderer = renderer::Renderer::new();

    renderer.run();
}
