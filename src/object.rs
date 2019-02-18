use nalgebra as na;

use na::Vector3;
use crate::math::{reflect, refract};

#[derive(Debug, Clone)]
pub struct Material {
    diffuse: Option<Diffuse>,
    specular: Option<Specular>,
    reflect: Option<f32>,
    refract: Option<Refract>,
}

#[derive(Debug, Clone)]
pub struct Diffuse {
    kind: DiffuseKind,
    albedo: f32,
}

#[derive(Debug, Clone)]
pub enum DiffuseKind {
    Color([f32; 3]),
}

#[derive(Debug, Clone, Copy)]
pub struct Specular {
    specular_exp: f32,
    albedo: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Refract {
    index: f32,
    albedo: f32,
}

impl Material {
    pub const fn none() -> Self {
        Self {
            diffuse: None,
            specular: None,
            reflect: None,
            refract: None,
        }
    }

    pub const fn color(diffuse: [f32; 3], albedo: f32) -> Self {
        Self {
            diffuse: Some(Diffuse { kind: DiffuseKind::Color(diffuse), albedo }),
            specular: None,
            reflect: None,
            refract: None,
        }
    }

    pub const fn with_specular(self, specular_exp: f32, albedo: f32) -> Self {
        Self {
            specular: Some(Specular { specular_exp, albedo }),
            ..self
        }
    }

    pub const fn with_reflect(self, albedo: f32) -> Self {
        Self {
            reflect: Some(albedo),
            ..self
        }
    }

    pub const fn with_refract(self, index: f32, albedo: f32) -> Self {
        Self {
            refract: Some(Refract { index, albedo }),
            ..self
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

pub trait Object: Sync {
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
pub struct Checkerboard {
    origin: Vector3<f32>,
    cell_dir: (Vector3<f32>, Vector3<f32>),
    dims: (u32, u32),
    material: (Material, Material),
}

impl Checkerboard {
    pub fn new(
        origin: Vector3<f32>,
        cell_dir: (Vector3<f32>, Vector3<f32>),
        dims: (u32, u32),
        material: (Material, Material),
    ) -> Self {
        Self {
            origin,
            cell_dir,
            dims,
            material,
        }
    }

    fn normal(&self) -> Vector3<f32> {
        self.cell_dir.0.cross(&self.cell_dir.1).normalize()
    }
}

impl Object for Checkerboard {
    fn ray_intersect(&self, orig: Vector3<f32>, dir: Vector3<f32>) -> Option<IntersectionInfo> {
        let p = orig - self.origin;
        let n = self.normal();
        let dir = dir.normalize();
        let neg_dist = n.dot(&p) / n.dot(&dir);
        if neg_dist.is_sign_positive() {
            return None;
        }

        let hit = p - neg_dist * dir;
        let len_0 = hit.dot(&self.cell_dir.0) / self.cell_dir.0.dot(&self.cell_dir.0);
        let len_1 = hit.dot(&self.cell_dir.1) / self.cell_dir.1.dot(&self.cell_dir.1);
        if len_0 < 0.0 || len_1 < 0.0 || len_0 >= self.dims.0 as f32 || len_1 >= self.dims.1 as f32 {
            return None;
        }
        let parity = len_0 as u32 + len_1 as u32;

        let hit = hit + self.origin;
        let material = if parity % 2 == 0 {
            self.material.0.clone()
        } else {
            self.material.1.clone()
        };

        Some(IntersectionInfo {
            dist: -neg_dist,
            hit,
            normal: n,
            material,
        })
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

const AIR_REFRACTION_INDEX: f32 = 1.0;

pub fn test_scene_intersect(
    orig: Vector3<f32>,
    dir: Vector3<f32>,
    objects: &[&dyn Object],
) -> Option<IntersectionInfo> {
    let mut intersections: Vec<_> = objects
        .iter()
        .filter_map(move |object| object.ray_intersect(orig, dir))
        .collect();
    intersections.sort_unstable_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap());
    intersections.pop()
}

pub fn render_scene(
    orig: Vector3<f32>,
    dir: Vector3<f32>,
    objects: &[&dyn Object],
    lights: &[Light],
    recursion_limit: u32,
) -> [f32; 3] {
    if recursion_limit == 0 { None } else { Some(()) }
        .and_then(|_| test_scene_intersect(orig, dir, objects))
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
                    let shadow_info = test_scene_intersect(shadow_orig, light_dir, objects);
                    match &shadow_info {
                        Some(shadow_info) if shadow_info.dist < light_dist => None,
                        _ => Some((light_dir, light)),
                    }
                })
                .collect();

            let diffuse_color_vec =
                if let Some(Diffuse { kind, albedo }) = &info.material.diffuse {
                    let diffuse_intensity: f32 = filtered_lights
                        .iter()
                        .map(|(light_dir, light)| {
                            light.intensity * f32::max(0.0, light_dir.dot(&info.normal))
                        })
                        .sum();
                    let raw_diffuse_color = match kind {
                        DiffuseKind::Color(diffuse) => diffuse.clone(),
                    };
                    Vector3::from(raw_diffuse_color) * diffuse_intensity * *albedo
                } else {
                    Vector3::from([0.0, 0.0, 0.0])
                };
            let specular_color_vec =
                if let Some(Specular { specular_exp, albedo }) = info.material.specular {
                    let specular_intensity: f32 = filtered_lights
                        .iter()
                        .map(|(light_dir, light)| {
                            let reflect_dir = reflect(light_dir.clone(), info.normal);
                            let angle = f32::max(0.0, reflect_dir.dot(&dir));
                            light.intensity * f32::powf(angle, specular_exp)
                        })
                        .sum();
                    Vector3::from([1.0, 1.0, 1.0]) * specular_intensity * albedo
                } else {
                    Vector3::from([0.0, 0.0, 0.0])
                };
            let reflect_color_vec =
                if let Some(albedo_reflect) = info.material.reflect {
                    let reflect_dir = reflect(dir, info.normal);
                    let reflect_orig = if reflect_dir.dot(&info.normal).is_sign_negative() {
                        info.hit - info.normal * 1e-3
                    } else {
                        info.hit + info.normal * 1e-3
                    };
                    let raw_reflect_color = render_scene(
                        reflect_orig,
                        reflect_dir,
                        objects,
                        lights,
                        recursion_limit - 1,
                    );
                    Vector3::from(raw_reflect_color) * albedo_reflect
                } else {
                    Vector3::from([0.0, 0.0, 0.0])
                };
            let refract_color_vec =
                if let Some(Refract { index, albedo }) = info.material.refract {
                    let refract_dir = refract(dir, info.normal, AIR_REFRACTION_INDEX, index);
                    let refract_orig = if refract_dir.dot(&info.normal).is_sign_negative() {
                        info.hit - info.normal * 1e-3
                    } else {
                        info.hit + info.normal * 1e-3
                    };
                    let raw_refract_color = render_scene(
                        refract_orig,
                        refract_dir,
                        objects,
                        lights,
                        recursion_limit - 1,
                    );
                    Vector3::from(raw_refract_color) * albedo
                } else {
                    Vector3::from([0.0, 0.0, 0.0])
                };
            let mut color_vec =
                diffuse_color_vec +
                specular_color_vec +
                reflect_color_vec +
                refract_color_vec;
            let max = color_vec.max();
            if max > 1.0 {
                color_vec /= max;
            }
            Some(color_vec.into())
        })
        .unwrap_or([0.2, 0.7, 0.8])
}
