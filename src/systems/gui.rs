use specs::{System, WriteStorage, ReadStorage, Read, Entities, Write, Entity};
use bracket_lib::prelude::VirtualKeyCode;
use crate::components::gui::*;
use crate::state::{PortableContext, WorldState, CurrentWorldState};
use crate::components::basic::{Actor, Position};
use crate::components::tag::PlayerTag;

pub struct GUIUpdate;

impl <'a> System <'a> for GUIUpdate {
    type SystemData = (
        ReadStorage <'a, Actor>,
        ReadStorage <'a, Position>,
        ReadStorage <'a, PlayerTag>,
        ReadStorage <'a, DebugInfoBox>,
        WriteStorage <'a, PlayerCard>,
        WriteStorage <'a, Panel>,
        WriteStorage <'a, TextBox>,
        Read <'a, PortableContext>,
        Write <'a, CurrentWorldState>,
        Entities<'a>
    );

    fn run (&mut self, (actors, positions, player_tag, debuginfo, mut player_card, mut panels, mut textboxes, ctx, mut wrld_state, entities) : Self::SystemData) {
        use specs::Join;

        //input for palyer card
        match ctx.key {
            Some(VirtualKeyCode::Tab) => {
                if wrld_state.0 != WorldState::TextBoxFocused {
                    for (panel, card) in (&mut panels, &mut player_card).join() {
                        card.cycle_alignment(panel, ctx.screen_size.0);
                    } 
                }
            },
            _ => {}
        }
        //update playercard
        for (_card, textbx) in (&player_card, &mut textboxes).join() {
            for (actor, _player) in (&actors, &player_tag).join() {
                let player_info = format!("
                        HP:{}/{} \\n
                        \\n
                        STR:{} \\n
                        DEX:{} \\n
                        CON:{} \\n
                        WIS:{} \\n
                        INT:{} \\n
                ", actor.current_health, actor.max_health, actor.strength, actor.dexterity, actor.constitution, actor.wisdom, actor.intelligence);
                textbx.update_text(player_info);
            }
        }

        //update debug info box
        for (_debugbox, textbx) in (&debuginfo, &mut textboxes).join() {
            let mut player_x = 0;
            let mut player_y = 0;
            for (position, _player) in (&positions, &player_tag).join() {
                player_x = position.x;
                player_y = position.y;
            }
            let player_info = format!("
                X : {}     Y: {} \\n
                FPS : {} \\n
            ", player_x, player_y, ctx.fps);
            textbx.update_text(player_info);
        }

        //update animated textboxes
        for textbx in (&mut textboxes).join() {
            if textbx.is_animated && !textbx.done_animating {
                textbx.animate_step(ctx.delta);
            }
        }
    }
}