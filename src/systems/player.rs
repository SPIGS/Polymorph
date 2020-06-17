use specs::{System, ReadStorage, WriteStorage, Read, Entities};
use bracket_lib::prelude::VirtualKeyCode;
use crate::components::basic::*;
use crate::components::tag::PlayerTag;
use crate::state::PortableContext;
use crate::raw::*;

pub struct PickUpSystem;

impl <'a> System <'a> for PickUpSystem {
    type SystemData = (
        ReadStorage <'a, Position>,
        ReadStorage <'a, ItemWrapper>,
        ReadStorage <'a, PlayerTag>,
        ReadStorage <'a, Currency>,
        WriteStorage <'a, Inventory>,
        Read <'a, PortableContext>,
        Entities<'a>
    );

    fn run (&mut self, (positions, item_wrappers, player_tag, currencies, mut inventory, ctx, entities) : Self::SystemData) {
        use specs::Join;

        match ctx.key {
            Some(VirtualKeyCode::G) => {
                
                //get player coords
                let mut player_x = 0;
                let mut player_y = 0;
                for (_player, position) in (&player_tag, &positions).join() {
                    player_x = position.x;
                    player_y = position.y;
                }

                //try to get the item data
                let mut item_data : Option<ItemRaw> = Option::None;
                let mut currency_amt = 0;
                for (position, item_wrapper, e) in (&positions, &item_wrappers, &entities).join() {
                    if player_x == position.x && player_y == position.y {
                        //get the item wrapper data
                        item_data = Option::from(item_wrapper.item_data.clone());
                        
                        //if it's a currency get its amount too
                        let value_option = currencies.get(e);
                        match value_option {
                            Some(currency) => {
                                currency_amt = currency.amt;
                            },
                            None => {}
                        }

                        // delete the entity
                        let delete_result = entities.delete(e);
                        match delete_result {
                            Ok(_t) => {
                                info!("Picked up item \"{}\" at position {},{}", item_wrapper.item_data.name, position.x, position.y);
                            },
                            Err(e) => {
                                error!("{}", e);
                                error!("Error deleting item \"{}\" entity on pick up at postion {}, {}", item_wrapper.item_data.name, position.x, position.y);
                            },
                        }
                    }
                }

                //if there is item data do something with it
                match item_data {
                    Some(item_raw) => {
                        // if the item is a currency add it to the total currency, else add it to the wallet
                        match item_raw.item_type.as_str() {
                            "currency" => {
                                 for (_player, inventory) in (&player_tag, &mut inventory).join() {
                                    inventory.money += item_raw.value * currency_amt as f32;
                                }
                            },
                            _ => {
                                 for (_player, inventory) in (&player_tag, &mut inventory).join() {
                                    inventory.add_item(item_raw.clone());
                                }
                            },
                        }

                       
                    },
                    None => {},
                }
            },
            _ => {},
        }
        
    }
} 