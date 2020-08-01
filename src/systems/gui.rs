use specs::{System, WriteStorage, ReadStorage, Read, Entities};
use bracket_lib::prelude::VirtualKeyCode;
use crate::components::gui::*;
use crate::state::PortableContext;
use crate::components::basic::Actor;
use crate::components::tag::PlayerTag;

pub struct GUIUpdate;

impl <'a> System <'a> for GUIUpdate {
    type SystemData = (
        ReadStorage <'a, Actor>,
        ReadStorage <'a, PlayerTag>,
        WriteStorage <'a, PlayerCard>,
        WriteStorage <'a, Panel>,
        WriteStorage <'a, TextBox>,
        Read <'a, PortableContext>,
        Entities<'a>
    );

    fn run (&mut self, (actors, player_tag, mut player_card, mut panels, mut textboxes, ctx, entities) : Self::SystemData) {
        use specs::Join;


        //input for palyer card
        match ctx.key {
            Some(VirtualKeyCode::Tab) => {
                for (panel, card) in (&mut panels, &mut player_card).join() {
                    card.cycle_alignment(panel, ctx.screen_size.0);
                } 
            },
            Some(VirtualKeyCode::Space) => {
                for (textbx, e) in (&mut textboxes, &entities).join() {
                    if textbx.waiting_on_proceed {
                        textbx.proceed();
                    } else if textbx.waiting_on_close {
                        let del_result = entities.delete(e);
                        match del_result {
                            Ok(_t) => {
                                debug!("Textbox closed.");
                            },
                            Err(e) => {
                                error!("{}", e);
                                error!("Error closing textbox.");
                            },
                        }
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

        //update animated textboxes
        for textbx in (&mut textboxes).join() {
            if textbx.is_animated && !textbx.done_animating {
                textbx.animate_step(ctx.delta);
            }
        }
    }
}