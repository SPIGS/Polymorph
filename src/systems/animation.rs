use specs::{System, Read, WriteStorage};

use crate::state::PortableContext;
use crate::components::basic::{Renderable, ColorLerp, CycleAnimation};

pub struct AnimationSystem;

impl <'a> System <'a> for AnimationSystem {
    type SystemData = (
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ColorLerp>,
        WriteStorage<'a, CycleAnimation>,
        Read<'a, PortableContext>,
    );

    fn run (&mut self, (mut renderables, mut colorlerps, mut cycle_animations, ctx) : Self::SystemData) {
        use specs::Join;
        
        for (renderable, colorlerp) in (&mut renderables, &mut colorlerps).join() {
            colorlerp.lerp(ctx.delta);
            renderable.fg = colorlerp.get_current_color();
        }

        for (renderable, cycle_animation) in (&mut renderables, &mut cycle_animations).join() {
            cycle_animation.cycle(ctx.delta);
            renderable.glyph = cycle_animation.get_current_frame();
        }
    }

}