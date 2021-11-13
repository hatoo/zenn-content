fn main() {
    const SHADER_PATH: &str = env!("shader.spv");
    const SHADER: &[u8] = include_bytes!(env!("shader.spv"));

    dbg!(SHADER_PATH);
    dbg!(SHADER.len());
}
