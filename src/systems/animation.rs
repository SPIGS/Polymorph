use specs::{System, Read, ReadStorage, WriteStorage};

use crate::state::DeltaTime;
use crate::components::basic::{Renderable, ColorLerp, CycleAnimation};
use bracket_lib::prelude::RGB;

pub struct AnimationSystem;

impl <'a> System <'a> for AnimationSystem {
    type SystemData = (
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ColorLerp>,
        WriteStorage<'a, CycleAnimation>,
        Read<'a, DeltaTime>,
    );

    fn run (&mut self, (mut renderables, mut colorlerps, mut cycle_animations, delta) : Self::SystemData) {
        use specs::Join;
        
        for (renderable, colorlerp) in (&mut renderables, &mut colorlerps).join() {
            colorlerp.lerp(delta.0);
            renderable.fg = colorlerp.get_current_color();
        }

        for (renderable, cycle_animation) in (&mut renderables, &mut cycle_animations).join() {
            cycle_animation.cycle(delta.0);
            renderable.glyph = cycle_animation.get_current_frame();
        }
    }

}