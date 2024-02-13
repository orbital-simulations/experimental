use crate::Engine;

pub mod collision;
pub mod inclined_fall;
pub mod many_particles;
pub mod pendulum;
pub mod penetration;
pub mod resting;
pub mod simple_fall;
pub mod springs;

pub use collision::*;
pub use inclined_fall::*;
pub use many_particles::*;
pub use pendulum::*;
pub use penetration::*;
pub use resting::*;
pub use simple_fall::*;
pub use springs::*;

pub trait Scenario {
    fn name(&self) -> &str;

    fn create(&self) -> Engine;

    fn update(&self, _engine: &mut Engine) {}
}
