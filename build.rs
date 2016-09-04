extern crate vulkano_shaders;

fn main() {
    // building the shaders used in the examples
    vulkano_shaders::build_glsl_shaders([
        ("shaders/triangle_vs.glsl", vulkano_shaders::ShaderType::Vertex),
        ("shaders/triangle_fs.glsl", vulkano_shaders::ShaderType::Fragment),
        ("shaders/teapot_vs.glsl", vulkano_shaders::ShaderType::Vertex),
        ("shaders/teapot_fs.glsl", vulkano_shaders::ShaderType::Fragment),
        ("shaders/image_vs.glsl", vulkano_shaders::ShaderType::Vertex),
        ("shaders/image_fs.glsl", vulkano_shaders::ShaderType::Fragment),
    ].iter().cloned());
}
