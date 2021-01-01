use crate::Mat3;

#[derive(Clone, Debug)]
pub struct ModelComponent {
    pub name: String,
    pub transform: Mat3,
}
