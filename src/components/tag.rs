use specs::{NullStorage, Component};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct PlayerTag;