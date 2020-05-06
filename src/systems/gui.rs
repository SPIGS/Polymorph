use specs::{System, ReadStorage, WriteStorage, Read, Entities};
use bracket_lib::prelude::VirtualKeyCode;
use crate::components::basic::*;
use crate::components::gui::*;
use crate::state::CurrentInput;

pub struct GUIUpdate;

impl <'a> System <'a> for GUIUpdate {
    type SystemData = (
        WriteStorage <'a, PlayerCard>,
        Read <'a, CurrentInput>,
    );

    fn run (&mut self, (mut player_card, current_input) : Self::SystemData) {
        use specs::Join;
        
        for card in (&mut player_card).join() {
            match current_input.key {
                Some(VirtualKeyCode::Tab) => {
                    card.cycle_justification();
                },
                _ => {}
            }
        }
    }
}