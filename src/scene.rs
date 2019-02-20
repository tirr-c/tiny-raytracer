use nalgebra::Vector3;
use crate::{
    material::{Diffuse, DiffuseKind, Refract, Specular},
    math::{reflect, refract},
    object::{IntersectionInfo, Object},
};

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
        .map(|info| {
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
                    nalgebra::zero()
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
                    nalgebra::zero()
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
                    nalgebra::zero()
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
                    nalgebra::zero()
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
            color_vec.into()
        })
        .unwrap_or([0.2, 0.7, 0.8])
}
