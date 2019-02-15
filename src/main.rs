const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

fn main() -> Result<(), failure::Error> {
    let mut framebuffer = tiny_raytracer::Framebuffer::new(WIDTH, HEIGHT);

    let buf = framebuffer.buf_mut();
    let wf = WIDTH as f32;
    let hf = HEIGHT as f32;
    for r in 0..HEIGHT {
        for c in 0..WIDTH {
            buf[c + r * WIDTH] = [r as f32 / hf, c as f32 / wf, 0.0];
        }
    }

    let file = std::fs::File::create("output.png")?;
    framebuffer.write_png(file)?;
    Ok(())
}
