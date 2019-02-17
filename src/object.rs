use nalgebra as na;

use na::Vector3;

#[derive(Debug, Clone)]
pub struct Material {
    kind: MaterialKind,
    specular_exp: f32,
    albedo_diffuse: f32,
    albedo_specular: f32,
}

#[derive(Debug, Clone)]
pub enum MaterialKind {
    Color([f32; 3]),
}

impl Material {
    pub const fn color(
        diffuse: [f32; 3],
        specular_exp: f32,
        albedo_diffuse: f32,
        albedo_specular: f32,
    ) -> Self {
        Self {
            kind: MaterialKind::Color(diffuse),
            specular_exp,
            albedo_diffuse,
            albedo_specular,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntersectionInfo {
    pub dist: f32,
    pub hit: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub material: Material,
}

pub trait Object {
    fn ray_intersect(&self, orig: Vector3<f32>, dir: Vector3<f32>) -> Option<IntersectionInfo>;
}

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Vector3<f32>,
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(center: Vector3<f32>, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    fn material(&self) -> Material {
        self.material.clone()
    }
}

impl Object for Sphere {
    fn ray_intersect(&self, orig: Vector3<f32>, dir: Vector3<f32>) -> Option<IntersectionInfo> {
        let dir_1 = dir.normalize();
        let radius_sq = self.radius * self.radius;

        let vec_to_center = self.center - orig;
        let dir_len = vec_to_center.dot(&dir_1);
        let dist_to_line = vec_to_center.dot(&vec_to_center) - dir_len * dir_len;
        if dist_to_line > radius_sq {
            return None;
        }

        let segment_len = f32::sqrt(radius_sq - dist_to_line);
        let near = dir_len - segment_len;
        let far = dir_len + segment_len;

        let selected = if near.is_sign_negative() { far } else { near };
        if selected.is_sign_negative() {
            None
        } else {
            let hit = orig + dir_1 * selected;
            Some(IntersectionInfo {
                dist: selected,
                hit,
                normal: (hit - self.center).normalize(),
                material: self.material(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Light {
    position: Vector3<f32>,
    intensity: f32,
}

impl Light {
    pub fn new(position: Vector3<f32>, intensity: f32) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

fn reflect(a: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
    a - a.dot(&n) * 2.0 * n
}

pub fn scene_intersect(
    orig: Vector3<f32>,
    dir: Vector3<f32>,
    spheres: &[Sphere],
    lights: &[Light],
) -> Option<[f32; 3]> {
    let mut intersections: Vec<_> = spheres
        .iter()
        .filter_map(move |sphere| sphere.ray_intersect(orig, dir))
        .collect();
    intersections.sort_unstable_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap());
    intersections
        .pop()
        .map(|info| {
            let dir = dir.normalize();
            let diffuse_intensity: f32 = lights
                .iter()
                .map(|light| {
                    let light_dir = (light.position - info.hit).normalize();
                    light.intensity * f32::max(0.0, light_dir.dot(&info.normal))
                })
                .sum();
            let specular_intensity: f32 = lights
                .iter()
                .map(|light| {
                    let light_dir = (light.position - info.hit).normalize();
                    let reflect_dir = reflect(light_dir, info.normal);
                    let angle = f32::max(0.0, reflect_dir.dot(&dir));
                    light.intensity * f32::powf(angle, info.material.specular_exp)
                })
                .sum();
            match info.material.kind {
                MaterialKind::Color(diffuse) => {
                    let diffuse_color_vec =
                        Vector3::from(diffuse) *
                        diffuse_intensity *
                        info.material.albedo_diffuse;
                    let specular_color_vec =
                        Vector3::from([1.0, 1.0, 1.0]) *
                        specular_intensity *
                        info.material.albedo_specular;
                    let mut color_vec = diffuse_color_vec + specular_color_vec;
                    let max = color_vec.max();
                    if max > 1.0 {
                        color_vec /= max;
                    }
                    color_vec.into()
                },
            }
        })
}
