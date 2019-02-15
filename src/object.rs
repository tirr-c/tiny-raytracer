use nalgebra as na;

use na::Vector3;

pub trait Object {
    fn ray_intersect(&self, orig: Vector3<f32>, dir: Vector3<f32>) -> Option<Vector3<f32>>;
}

pub struct Sphere {
    center: Vector3<f32>,
    radius: f32,
}

impl Sphere {
    pub fn new(center: Vector3<f32>, radius: f32) -> Self {
        Self {
            center,
            radius,
        }
    }
}

impl Object for Sphere {
    fn ray_intersect(&self, orig: Vector3<f32>, mut dir: Vector3<f32>) -> Option<Vector3<f32>> {
        dir = dir.normalize();
        let radius_sq = self.radius * self.radius;

        let vec_to_center = self.center - orig;
        let dir_len = vec_to_center.dot(&dir);
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
            Some(orig + dir * selected)
        }
    }
}
