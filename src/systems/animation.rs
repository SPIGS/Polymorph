use specs::{System, Read, ReadStorage, WriteStorage};

use crate::state::DeltaTime;
use crate::components::basic::{Renderable, ColorLerp};
use bracket_lib::prelude::RGB;

pub struct AnimationSystem;

impl <'a> System <'a> for AnimationSystem {
    type SystemData = (
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ColorLerp>,
        Read<'a, DeltaTime>,
    );

    fn run (&mut self, (mut renderables, mut colorlerps, delta) : Self::SystemData) {
        use specs::Join;
        
        for (renderable, colorlerp) in (&mut renderables, &mut colorlerps).join() {
            colorlerp.lerp(delta.0);
            renderable.fg = colorlerp.get_current_color();
        }
    }

}