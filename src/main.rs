#[allow(dead_code)]
#[allow(unused_parens)]
#[macro_use]
extern crate vulkano;
extern crate winit;
extern crate vulkano_win;
extern crate cgmath;

mod renderer;

fn main() {
    let renderer = renderer::Renderer::new();

    renderer.run();
}
