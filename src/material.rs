#[derive(Debug, Clone)]
pub struct Material {
    pub(crate) diffuse: Option<Diffuse>,
    pub(crate) specular: Option<Specular>,
    pub(crate) reflect: Option<f32>,
    pub(crate) refract: Option<Refract>,
}

#[derive(Debug, Clone)]
pub struct Diffuse {
    pub(crate) kind: DiffuseKind,
    pub(crate) albedo: f32,
}

#[derive(Debug, Clone)]
pub enum DiffuseKind {
    Color([f32; 3]),
}

#[derive(Debug, Clone, Copy)]
pub struct Specular {
    pub(crate) specular_exp: f32,
    pub(crate) albedo: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Refract {
    pub(crate) index: f32,
    pub(crate) albedo: f32,
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
