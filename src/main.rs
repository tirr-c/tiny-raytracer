use nalgebra::Vector3;
use tiny_raytracer::{
    object::{Checkerboard, Sphere},
    Light,
    Material,
    Scene,
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
    let mut scene = Scene::new();

    scene.push_object(Sphere::new(Vector3::from([-3.0,  0.0, -16.0]), 2.0, IVORY));
    scene.push_object(Sphere::new(Vector3::from([-1.0, -1.5, -12.0]), 2.0, GLASS));
    scene.push_object(Sphere::new(Vector3::from([ 1.5, -0.5, -18.0]), 3.0, RED_RUBBER));
    scene.push_object(Sphere::new(Vector3::from([ 7.0,  5.0, -18.0]), 4.0, MIRROR));
    scene.push_object(
        Checkerboard::new(
            Vector3::from([-10.0, -4.0, -30.0]),
            (Vector3::from([0.0, 0.0, 2.0]), Vector3::from([2.0, 0.0, 0.0])),
            (10, 10),
            (CHECKER_WHITE, CHECKER_ORANGE),
        ),
    );

    scene.push_light(Light::new(Vector3::from([-20.0, 20.0,  20.0]), 1.5));
    scene.push_light(Light::new(Vector3::from([ 30.0, 50.0, -25.0]), 1.0));
    scene.push_light(Light::new(Vector3::from([ 30.0, 20.0,  30.0]), 1.7));

    scene.render(&mut framebuffer, WIDTH, HEIGHT, std::f32::consts::PI / 3.0);

    let file = std::fs::File::create("output.png")?;
    framebuffer.write_png(file)?;
    Ok(())
}
