fn main() {
    const SHADER_PATH: &str = env!("rasterization_example_shader.spv");
    const SHADER: &[u8] = include_bytes!(env!("rasterization_example_shader.spv"));

    dbg!(SHADER_PATH);
    dbg!(SHADER.len());
}
