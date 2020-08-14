use specs::{System, Read, Write, ReadStorage, WriteStorage};
use bracket_lib::prelude::VirtualKeyCode;
use crate::components::basic::{Position, Actor};
use crate::components::tag::PlayerTag;
use crate::state::PortableContext;
use crate::level_generation::map::{Map, VisibilityMap};
use bracket_lib::prelude::field_of_view;
use bracket_lib::prelude::Point;
use crate::state::{WorldState, CurrentWorldState};

pub struct PlayerMoveSystem;

impl <'a> System <'a> for PlayerMoveSystem {
    type SystemData = (
        ReadStorage <'a, PlayerTag>,
        WriteStorage <'a, Position>,
        WriteStorage <'a, Actor>,
        Read <'a, PortableContext>,
        Read <'a, CurrentWorldState>,
    );

    fn run (&mut self, (playertag, mut positions, mut _actors, ctx, wrld_state) : Self::SystemData) {
        use specs::Join;
            
        match wrld_state.0 {
            WorldState::NoAction => {
                for (_playertag, position) in (&playertag, &mut positions).join() {
                    match ctx.key {
                        Some(VirtualKeyCode::Up) => {position.y -= 1;},
                        Some(VirtualKeyCode::Down) => {position.y += 1},
                        Some(VirtualKeyCode::Left) => {position.x -= 1},
                        Some(VirtualKeyCode::Right) => {position.x += 1},
                        None => {},
                        _ => {},
                    }
                }
            },
            _ => {},
        }
    }

}

pub struct VisibilitySystem;

impl <'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadStorage <'a, PlayerTag>,
        ReadStorage <'a, Position>,
        Read <'a, Map>,
        Write<'a, VisibilityMap>,

    );

    fn run (&mut self, (playertag, positions, map, mut visibility_map) : Self::SystemData) {
        use specs::Join;

        for (_player, position) in (&playertag, &positions).join() {
            let pt = Point::from_tuple((position.x, position.y));
            visibility_map.reset_visible();
            visibility_map.write(field_of_view(pt, 40, &*map));
        }
    }
}