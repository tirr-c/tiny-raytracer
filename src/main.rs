use nalgebra as na;

use na::Vector3;
use rayon::prelude::*;
use tiny_raytracer::{object::{self, Sphere}, Material};

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

const IVORY: [f32; 3] = [0.4, 0.4, 0.3];
const RED_RUBBER: [f32; 3] = [0.3, 0.1, 0.1];

fn main() -> Result<(), failure::Error> {
    let mut framebuffer = tiny_raytracer::Framebuffer::new(WIDTH, HEIGHT);

    let spheres = [
        Sphere::new(Vector3::from([-3.0,  0.0, -16.0]), 2.0, IVORY),
        Sphere::new(Vector3::from([-1.0, -1.5, -12.0]), 2.0, RED_RUBBER),
        Sphere::new(Vector3::from([ 1.5, -0.5, -18.0]), 3.0, RED_RUBBER),
        Sphere::new(Vector3::from([ 7.0,  5.0, -18.0]), 4.0, IVORY),
    ];

    let wf = WIDTH as f32;
    let hf = HEIGHT as f32;
    let fov_half = std::f32::consts::PI / 4.0;
    let fov_tan = f32::tan(fov_half);

    framebuffer.render_with(|| {
        (0..(WIDTH * HEIGHT))
            .into_par_iter()
            .map(|rc| {
                let r = rc / WIDTH;
                let c = rc % WIDTH;
                let rf = r as f32;
                let cf = c as f32;
                let x = (2.0 * (cf + 0.5) / wf - 1.0) * fov_tan * wf / hf;
                let y = -(2.0 * (rf + 0.5) / hf - 1.0) * fov_tan;
                let dir = Vector3::from([x, y, -1.0]);
                let info = object::scene_intersect(na::zero(), dir, &spheres);
                if let Some(info) = info {
                    match info.material {
                        Material::Color { diffuse } => diffuse,
                    }
                } else {
                    [0.2, 0.7, 0.8]
                }
            })
            .collect()
    });

    let file = std::fs::File::create("output.png")?;
    framebuffer.write_png(file)?;
    Ok(())
}
