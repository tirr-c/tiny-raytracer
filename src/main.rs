use nalgebra as na;

use na::Vector3;
use tiny_raytracer::Object;

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

fn main() -> Result<(), failure::Error> {
    let mut framebuffer = tiny_raytracer::Framebuffer::new(WIDTH, HEIGHT);

    let sphere = tiny_raytracer::object::Sphere::new(Vector3::from([-3.0, 0.0, -16.0]), 2.0);

    let buf = framebuffer.buf_mut();
    let wf = WIDTH as f32;
    let hf = HEIGHT as f32;
    let fov_half = std::f32::consts::PI / 4.0;
    let fov_tan = f32::tan(fov_half);
    for r in 0..HEIGHT {
        let rf = r as f32;
        for c in 0..WIDTH {
            let cf = c as f32;
            let x = (2.0 * (cf + 0.5) / wf - 1.0) * fov_tan * wf / hf;
            let y = -(2.0 * (rf + 0.5) / hf - 1.0) * fov_tan;
            let dir = Vector3::from([x, y, -1.0]);
            buf[c + r * WIDTH] = if sphere.ray_intersect(na::zero(), dir).is_some() {
                [0.4, 0.4, 0.3]
            } else {
                [0.2, 0.7, 0.8]
            };
        }
    }

    let file = std::fs::File::create("output.png")?;
    framebuffer.write_png(file)?;
    Ok(())
}
