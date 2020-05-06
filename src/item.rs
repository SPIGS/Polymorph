//! Contains the DATA REPRESENTATIONS for various generic types of items

//? Potion Adjectives
//* swurling
//* oily
//* bubbling
//* viscous
//* milky
//* watery
//* sticky
//* cool
//* warm
//* aerated
//* metallic
//? Potion colors
//* red
//* orange
//* yellow
//* green
//* blue
//* violet
//* ----- Off colors ----
//* azure
//* fuschia
//* silver
//* gold
//* black
//* white
//* pink
//* brown

//? status effects
//* - Healing (healing over time)
//* Blind (reduces the visisble area)
//* Amnesia (forget the discovered parts of the map and/or discovered potions and/or scrolls)
//* - Night vision (opposite of blindess)
//* - Foresight (opposite of amnesia)
//* Poisoned (damage over time)
//* Burning (on fire)
//* - Flight (grants flight)
//* - Fire Resistance (resistance to fire)
//* - Invisiblility (invsible)
//* Paralyisis (cant move for time)
//* Frightened (idk how to implement this)
//* Deaf (hearing based abilities dont work and no music/sound)
//* Fatigued (stamina drain)

use specs::{Entity, EntityBuilder, Builder};
use crate::components::basic::{Renderable, Position, ItemWrapper, Currency};
use bracket_lib::prelude::RGB;
use crate::raw::*;

pub struct ItemBuilder;

impl ItemBuilder {
    pub fn build_item_with_name (entity : EntityBuilder, name : String, position : (i32, i32)) -> Entity {
        let id = RAW.lock().unwrap().get_item_id(name);
        let item = RAW.lock().unwrap().get_item(id).clone();
        return ItemBuilder::build_item(entity, item, position);
    }

    pub fn build_item_with_id (entity : EntityBuilder, id : u32, position : (i32, i32)) -> Entity {
        let item = RAW.lock().unwrap().get_item(id).clone();
        return ItemBuilder::build_item(entity, item, position);
    }

    fn build_item (entity : EntityBuilder, item : ItemRaw, position : (i32, i32)) -> Entity {
        let mut item_entity = entity;
        item_entity = item_entity.with(Position::new(position.0, position.1));
        match item.item_type.as_str() {
            "potion" => {
                item_entity = item_entity.with(Renderable::new(235, RGB::from_u8(0, 0, 255), RGB::from_u8(0, 0, 0), true));
            },
            "currency" => {
                item_entity = item_entity.with(Currency{amt : 1});
                item_entity = ItemBuilder::build_renderable(item_entity, item.clone());
            },
            _ => {
                item_entity = ItemBuilder::build_renderable(item_entity, item.clone());
            },
        }

        item_entity = item_entity.with(ItemWrapper::new(item.clone()));
        return item_entity.build();
    }

    fn build_renderable (builder : EntityBuilder, raw_item : ItemRaw) -> EntityBuilder {
        let mut temp_builder = builder;
        match raw_item.renderable {
            Some(r) => {
                //handle error for invalid foreground color:
                let foreground_result = RGB::from_hex(r.fg);
                let mut fg = RGB::from_u8(255, 0, 0);
                match foreground_result {
                    Ok(t) => {
                        fg = t;
                    },
                    Err(_e) => {
                        error!("Invalid RGB foreground for item \"{}\"; falling back on default color", raw_item.name);
                    },
                }
                //handle error for invalid background color:
                let background_result = RGB::from_hex(r.bg);
                let mut bg = RGB::from_u8(0, 0, 0);
                match background_result {
                    Ok(t) => {
                        bg = t;
                    },
                    Err(_e) => {
                        error!("Invalid RGB background for item \"{}\"; falling back on default color", raw_item.name);
                    },
                }
                //handle error for invalid glyph:
                let glyph_raw = r.character_code;
                let mut glyph = '!' as u16;
                if glyph_raw > 255 {
                    error!("Invalid character code for renderable for item \"{}\"; Falling back on default", raw_item.name);
                } else {
                    glyph = glyph_raw as u16;
                }
                
                temp_builder = temp_builder.with(Renderable::new(glyph, fg, bg, true));
            },
            _ => {
                temp_builder = temp_builder.with(Renderable::new_from_char('!', RGB::from_u8(255, 0, 0), RGB::from_u8(0, 0, 0), true));
                error!("Item: \"{}\" missing renderable component; Failing back on defaults", raw_item.name);
            }
        }
        return temp_builder;
    }
}



