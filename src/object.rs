use nalgebra as na;

use na::Vector3;

#[derive(Debug, Clone)]
pub struct Material {
    kind: MaterialKind,
    specular_exp: f32,
    albedo_diffuse: f32,
    albedo_specular: f32,
    albedo_reflect: f32,
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
        albedo_reflect: f32,
    ) -> Self {
        Self {
            kind: MaterialKind::Color(diffuse),
            specular_exp,
            albedo_diffuse,
            albedo_specular,
            albedo_reflect,
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

pub fn test_scene_intersect(
    orig: Vector3<f32>,
    dir: Vector3<f32>,
    spheres: &[Sphere],
) -> Option<IntersectionInfo> {
    let mut intersections: Vec<_> = spheres
        .iter()
        .filter_map(move |sphere| sphere.ray_intersect(orig, dir))
        .collect();
    intersections.sort_unstable_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap());
    intersections.pop()
}

pub fn render_scene(
    orig: Vector3<f32>,
    dir: Vector3<f32>,
    spheres: &[Sphere],
    lights: &[Light],
    recursion_limit: u32,
) -> [f32; 3] {
    let intersection = test_scene_intersect(orig, dir, spheres);
    intersection
        .and_then(|info| {
            let dir = dir.normalize();
            let filtered_lights: Vec<_> = lights
                .iter()
                .filter_map(|light| {
                    let raw_light_dir = light.position - info.hit;
                    let light_dir = raw_light_dir.normalize();
                    let light_dist = raw_light_dir.norm();

                    let shadow_orig = if light_dir.dot(&info.normal).is_sign_negative() {
                        info.hit - info.normal * 1e-3
                    } else {
                        info.hit + info.normal * 1e-3
                    };
                    let shadow_info = test_scene_intersect(shadow_orig, light_dir, spheres);
                    match &shadow_info {
                        Some(shadow_info) if shadow_info.dist < light_dist => None,
                        _ => Some((light_dir, light)),
                    }
                })
                .collect();

            let diffuse_intensity: f32 = filtered_lights
                .iter()
                .map(|(light_dir, light)| {
                    light.intensity * f32::max(0.0, light_dir.dot(&info.normal))
                })
                .sum();
            let specular_intensity: f32 = filtered_lights
                .iter()
                .map(|(light_dir, light)| {
                    let reflect_dir = reflect(light_dir.clone(), info.normal);
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
                    let raw_reflect_color = if recursion_limit == 0 {
                        return None;
                    } else {
                        let reflect_dir = reflect(dir, info.normal);
                        let reflect_orig = if reflect_dir.dot(&info.normal).is_sign_negative() {
                            info.hit - info.normal * 1e-3
                        } else {
                            info.hit + info.normal * 1e-3
                        };
                        render_scene(reflect_orig, reflect_dir, spheres, lights, recursion_limit - 1)
                    };
                    let reflect_color_vec =
                        Vector3::from(raw_reflect_color) *
                        info.material.albedo_reflect;
                    let mut color_vec = diffuse_color_vec + specular_color_vec + reflect_color_vec;
                    let max = color_vec.max();
                    if max > 1.0 {
                        color_vec /= max;
                    }
                    Some(color_vec.into())
                },
            }
        })
        .unwrap_or([0.2, 0.7, 0.8])
}
