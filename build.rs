extern crate vulkano_shaders;

fn main() {
    vulkano_shaders::build_glsl_shaders([("shaders/main_vs.glsl",
                                          vulkano_shaders::ShaderType::Vertex),
                                         ("shaders/main_fs.glsl",
                                          vulkano_shaders::ShaderType::Fragment)]
        .iter()
        .cloned());
}
