mod error;
mod framebuffer;
mod material;
mod math;
pub mod object;
mod scene;

pub use error::RenderError;
pub use framebuffer::Framebuffer;
pub use material::Material;

pub use scene::{Light, render_scene};
