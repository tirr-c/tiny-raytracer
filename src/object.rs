use nalgebra as na;

use na::Vector3;

#[derive(Debug, Clone)]
pub enum Material {
    Color {
        diffuse: [f32; 3],
    },
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
    diffuse: [f32; 3],
}

impl Sphere {
    pub fn new(center: Vector3<f32>, radius: f32, diffuse: [f32; 3]) -> Self {
        Self {
            center,
            radius,
            diffuse,
        }
    }

    fn material(&self) -> Material {
        Material::Color {
            diffuse: self.diffuse,
        }
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

pub fn scene_intersect(orig: Vector3<f32>, dir: Vector3<f32>, spheres: &[Sphere]) -> Option<IntersectionInfo> {
    let mut intersections: Vec<_> = spheres
        .iter()
        .filter_map(move |sphere| sphere.ray_intersect(orig, dir))
        .collect();
    intersections.sort_unstable_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap());
    intersections.pop()
}
