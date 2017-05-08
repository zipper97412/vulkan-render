
use specs::*;

#[derive(Clone, Debug)]
pub struct CompPosition(pub i32, pub i32);
impl Component for CompPosition {
    type Storage = VecStorage<CompPosition>;
}