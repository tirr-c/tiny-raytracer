use nalgebra::Vector3;
use rayon::prelude::*;
use tiny_raytracer::{
    object::{Checkerboard, Object, Sphere},
    Light,
    Material,
    render_scene,
};

const WIDTH: usize = 1024;
const HEIGHT: usize = 768;

const IVORY: Material = Material::color([0.4, 0.4, 0.3], 0.6).with_specular(50.0, 0.3).with_reflect(0.1);
const RED_RUBBER: Material = Material::color([0.3, 0.1, 0.1], 0.9).with_specular(10.0, 0.1);
const MIRROR: Material = Material::none().with_specular(1425.0, 10.0).with_reflect(0.8);
const GLASS: Material = Material::none().with_specular(125.0, 0.5).with_reflect(0.1).with_refract(1.5, 0.8);
const CHECKER_WHITE: Material = Material::color([1.0, 1.0, 1.0], 0.4);
const CHECKER_ORANGE: Material = Material::color([1.0, 0.7, 0.3], 0.4);

fn main() -> Result<(), failure::Error> {
    let mut framebuffer = tiny_raytracer::Framebuffer::new(WIDTH, HEIGHT);

    let objects: &[&dyn Object] = &[
        &Sphere::new(Vector3::from([-3.0,  0.0, -16.0]), 2.0, IVORY),
        &Sphere::new(Vector3::from([-1.0, -1.5, -12.0]), 2.0, GLASS),
        &Sphere::new(Vector3::from([ 1.5, -0.5, -18.0]), 3.0, RED_RUBBER),
        &Sphere::new(Vector3::from([ 7.0,  5.0, -18.0]), 4.0, MIRROR),
        &Checkerboard::new(
            Vector3::from([-10.0, -4.0, -30.0]),
            (Vector3::from([0.0, 0.0, 2.0]), Vector3::from([2.0, 0.0, 0.0])),
            (10, 10),
            (CHECKER_WHITE, CHECKER_ORANGE),
        ),
    ];
    let lights = [
        Light::new(Vector3::from([-20.0, 20.0,  20.0]), 1.5),
        Light::new(Vector3::from([ 30.0, 50.0, -25.0]), 1.0),
        Light::new(Vector3::from([ 30.0, 20.0,  30.0]), 1.7),
    ];

    let wf = WIDTH as f32;
    let hf = HEIGHT as f32;
    let fov_half = std::f32::consts::PI / 6.0;
    let fov_tan = f32::tan(fov_half);

    framebuffer.render_with(|| {
        (0..(WIDTH * HEIGHT))
            .into_par_iter()
            .map(|rc| {
                let r = rc / WIDTH;
                let c = rc % WIDTH;
                let rf = r as f32;
                let cf = c as f32;
                let dir_x = (cf + 0.5) - wf / 2.0;
                let dir_y = -(rf + 0.5) + hf / 2.0;
                let dir_z = -hf / (2.0 * fov_tan);
                let dir = Vector3::from([dir_x, dir_y, dir_z]);
                render_scene(nalgebra::zero(), dir, objects, &lights, 4)
            })
            .collect()
    });

    let file = std::fs::File::create("output.png")?;
    framebuffer.write_png(file)?;
    Ok(())
}
