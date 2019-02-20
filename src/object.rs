use nalgebra::Vector3;
use crate::{
    material::{Diffuse, DiffuseKind, Material, Refract, Specular},
    math::{reflect, refract},
};

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
