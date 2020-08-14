use specs::{System, WriteStorage, ReadStorage, Read, Entities, Write, Entity};
use bracket_lib::prelude::{VirtualKeyCode, TextBuilder};
use crate::components::gui::*;
use crate::state::{PortableContext, WorldState, CurrentWorldState};
use crate::components::basic::{Actor, Position};
use crate::components::tag::PlayerTag;
use crate::state::input::get_char_from_keypress;

pub struct GUIUpdate;

impl <'a> System <'a> for GUIUpdate {
    type SystemData = (
        ReadStorage <'a, Actor>,
        ReadStorage <'a, Position>,
        ReadStorage <'a, PlayerTag>,
        ReadStorage <'a, DebugInfoBox>,
        WriteStorage <'a, TextEntry>,
        WriteStorage <'a, PlayerCard>,
        WriteStorage <'a, Panel>,
        WriteStorage <'a, TextBox>,
        Read <'a, PortableContext>,
        Write <'a, CurrentWorldState>,
        Entities<'a>
    );

    fn run (&mut self, (actors, positions, player_tag, debuginfo, mut text_entries, mut player_card, mut panels, mut textboxes, ctx, mut wrld_state, entities) : Self::SystemData) {
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
            Some(VirtualKeyCode::Space) => {
                let mut entity : Option<Entity> = Option::None;
                for (textbx, textbox_entity) in (&mut textboxes, &entities).join() {
                    if textbx.waiting_on_proceed {
                        textbx.proceed();
                    }
                    if textbx.waiting_on_close && textbx.close_on_end {
                        entity = Option::from(textbox_entity);
                    }
                }
                match entity {
                    Some(e) => {
                        let delete_result = entities.delete(e);
                        textboxes.remove(e);
                        wrld_state.0 = WorldState::NoAction;
                        match delete_result {
                            Ok(_t) => { 
                                debug!("Textbox closed.");
                            },
                            Err(e) => {
                                error!("Error closing textbox.");
                                error!("{}", e);
                            },
                        }       
                    },
                    None => {},
                }
            },
            _ => {}
        }
        //update playercard
        for (_card, textbx) in (&player_card, &mut textboxes).join() {
            for (actor, _player) in (&actors, &player_tag).join() {
                let mut player_info = TextBuilder::empty();
                player_info.line_wrap(&format!("HP:{}/{}", actor.current_health, actor.max_health));
                player_info.ln().ln();
                player_info.line_wrap(&format!("STR:{}", actor.strength));
                player_info.ln();
                player_info.line_wrap(&format!("DEX:{}", actor.dexterity));
                player_info.ln();
                player_info.line_wrap(&format!("CON:{}", actor.constitution));
                player_info.ln();
                player_info.line_wrap(&format!("WIS:{}", actor.wisdom));
                player_info.ln();
                player_info.line_wrap(&format!("INT:{}", actor.intelligence));
                player_info.ln();
                player_info.line_wrap(&format!("CHA:"));
                textbx.force_update_buffer(player_info);
            }
        }

        //update debug info box
        for (debugbox, textbx) in (&debuginfo, &mut textboxes).join() {
            let mut player_x = 0;
            let mut player_y = 0;
            for (position, _player) in (&positions, &player_tag).join() {
                player_x = position.x;
                player_y = position.y;
            }

            let mut debug_info = TextBuilder::empty();
            debug_info.line_wrap(&format!("X:{};", player_x)).ln();
            debug_info.line_wrap(&format!("Y:{};", player_y)).ln();
            debug_info.line_wrap(&format!("Seed: {}", debugbox.seed.clone())).ln();
            debug_info.line_wrap(&format!("FPS:{}", ctx.fps));

            textbx.force_update_buffer(debug_info);
        }

        //update other textboxes
        for textbx in (&mut textboxes).join() {
            
            // animate
            if textbx.is_animated && !textbx.waiting_on_proceed && !textbx.waiting_on_close {
                textbx.animate_step(ctx.delta);
            }

            //focus
            if textbx.is_focused {
                wrld_state.0 = WorldState::TextBoxFocused;
            }

        }

        //update text entries
        for (textbx, entry) in (&mut textboxes, &mut text_entries).join() {
            match ctx.key {
                Some(k) => {
                    
                    if k == VirtualKeyCode::Back {
                        if entry.text.len() >= 1 {
                            entry.text.remove(entry.text.len() - 1);
                        }
                        let mut new_buffer = TextBuilder::empty();
                        new_buffer.append(&entry.text);
                        textbx.force_update_buffer(new_buffer);
                    } else {
                        if entry.text.len() < entry.max_length {
                            let character = get_char_from_keypress(k, ctx.shift);
                            if character != '\0' {
                                entry.text.push(character);
                                let mut new_buffer = TextBuilder::empty();
                                new_buffer.append(&entry.text);
                                textbx.force_update_buffer(new_buffer);
                            }
                        }
                    }
                },
                None => {},
            }
        }

    }
}