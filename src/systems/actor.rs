use specs::{System, Read, ReadStorage, WriteStorage};
use bracket_lib::prelude::VirtualKeyCode;
use crate::components::basic::{Position, Actor};
use crate::components::tag::PlayerTag;
use crate::state::PortableContext;

pub struct PlayerMoveSystem;

impl <'a> System <'a> for PlayerMoveSystem {
    type SystemData = (
        ReadStorage <'a, PlayerTag>,
        WriteStorage <'a, Position>,
        WriteStorage <'a, Actor>,
        Read <'a, PortableContext>,
    );

    fn run (&mut self, (playertag, mut positions, mut _actors, ctx) : Self::SystemData) {
        use specs::Join;
            
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
    }

}