use specs::{System, WriteStorage, Read};
use bracket_lib::prelude::VirtualKeyCode;
use crate::components::gui::*;
use crate::state::PortableContext;

pub struct GUIUpdate;

impl <'a> System <'a> for GUIUpdate {
    type SystemData = (
        WriteStorage <'a, PlayerCard>,
        Read <'a, PortableContext>,
    );

    fn run (&mut self, (mut player_card, ctx) : Self::SystemData) {
        use specs::Join;
        
        for card in (&mut player_card).join() {
            match ctx.key {
                Some(VirtualKeyCode::Tab) => {
                    card.cycle_justification();
                },
                _ => {}
            }
        }
    }
}