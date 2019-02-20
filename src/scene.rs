use nalgebra::Vector3;
use crate::{
    framebuffer::Framebuffer,
    material::{Diffuse, DiffuseKind, Refract, Specular},
    math::{reflect, refract},
    object::{IntersectionInfo, Object},
};

const AIR_REFRACTION_INDEX: f32 = 1.0;

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

#[derive(Default)]
pub struct Scene {
    objects: Vec<Box<dyn Object + Sync>>,
    lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_object<T: Object + 'static>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }

    pub fn push_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    fn test_intersect(&self, orig: Vector3<f32>, dir: Vector3<f32>) -> Option<IntersectionInfo> {
        let mut intersections: Vec<_> = self
            .objects
            .iter()
            .filter_map(move |object| object.ray_intersect(orig, dir))
            .collect();
        intersections.sort_unstable_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap());
        intersections.pop()
    }

    pub fn cast_ray(
        &self,
        orig: Vector3<f32>,
        dir: Vector3<f32>,
        recursion_limit: u32,
    ) -> [f32; 3] {
        if recursion_limit == 0 { None } else { Some(()) }
            .and_then(|_| self.test_intersect(orig, dir))
            .map(|info| {
                let dir = dir.normalize();
                let filtered_lights: Vec<_> = self
                    .lights
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
                        let shadow_info = self.test_intersect(shadow_orig, light_dir);
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
                        let raw_reflect_color = self.cast_ray(
                            reflect_orig,
                            reflect_dir,
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
                        let raw_refract_color = self.cast_ray(
                            refract_orig,
                            refract_dir,
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

    pub fn render(
        &self,
        fb: &mut Framebuffer,
        width: usize,
        height: usize,
        fov: f32,
    ) -> Framebuffer {
        use rayon::prelude::*;

        let wf = width as f32;
        let hf = height as f32;
        let fov_half = fov / 2.0;
        let fov_half_tan = f32::tan(fov_half);

        let old = fb.render_with(|| {
            (0..(width * height))
                .into_par_iter()
                .map(|rc| {
                    let r = rc / width;
                    let c = rc % width;
                    let rf = r as f32;
                    let cf = c as f32;
                    let dir_x = (cf + 0.5) - wf / 2.0;
                    let dir_y = -(rf + 0.5) + hf / 2.0;
                    let dir_z = -hf / (2.0 * fov_half_tan);
                    let dir = Vector3::from([dir_x, dir_y, dir_z]);
                    self.cast_ray(nalgebra::zero(), dir, 4)
                })
                .collect()
        });
        std::mem::replace(fb, old)
    }
}
