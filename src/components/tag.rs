use specs::{NullStorage, Component};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct PlayerTag;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct DirtyTag;