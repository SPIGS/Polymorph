use specs::{Component, NullStorage};

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct PlayerTag;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct TileTag;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct LookCursorTag;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct DirtyFlag;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct AlwaysLitFlag;

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct StaticFlag;