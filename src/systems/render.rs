use specs::{System, ReadStorage, Read};
use bracket_lib::prelude::DrawBatch;
use bracket_lib::prelude::Point;
use bracket_lib::prelude::ColorPair;
use bracket_lib::prelude::{RGB, HSV};
use bracket_lib::prelude::Rect;
use object_pool::Reusable;

use crate::components::basic::{Position, Renderable, Inventory, Actor};
use crate::components::gui::{PlayerCard, Justification, Panel};
use crate::components::tag::PlayerTag;
use crate::raw::RAW;

use crate::level_generation::map::VisibilityMap;

///Returns true if the given screen coords are on screen.
pub fn is_on_screen (screen_coords:(i32,i32), screen_size: (i32,i32)) -> bool {
	let mut on_screen = true;
	if ((screen_coords.0 < 0) || (screen_coords.0 > screen_size.0)) || ((screen_coords.1 < 0) || (screen_coords.1 > screen_size.1)) {
		on_screen = false;
	}
	return on_screen;
}

#[derive(Debug, PartialEq)]
pub enum ObjectShader {
    NoShading,
    Foreground,
    Background,
}

pub struct RenderSystem {
    pub draw_batch : Reusable<'static, DrawBatch>,
    pub horiz_offset : i32,
    pub vert_offset : i32,
    pub screen_size : (u32, u32),
}

impl RenderSystem {
    pub fn new (draw_batch : Reusable<'static, DrawBatch>, screen_size : (u32, u32)) -> Self {
        RenderSystem {
            draw_batch : draw_batch,
            horiz_offset : -(screen_size.0 as i32/8),
            vert_offset : 0,
            screen_size : screen_size,
        }
    }
}

impl <'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage <'a, Position>,
        ReadStorage <'a, Renderable>,
        ReadStorage <'a, PlayerTag>,
        ReadStorage <'a, Inventory>,
        ReadStorage <'a, Actor>,
        ReadStorage <'a, PlayerCard>,
        Read <'a, VisibilityMap>,
    );

    fn run (&mut self, (positions, renderables, player_tag, inventory, actors, player_card, visibility_map) : Self::SystemData) {
        use specs::Join;
        self.draw_batch.target(0);
        self.draw_batch.cls();

        //get offset from gui
        for card in (&player_card).join() {
            self.horiz_offset = match card.justification {
                Justification::RIGHT => -(self.screen_size.0 as i32/8),
                Justification::LEFT => (self.screen_size.0 as i32/8),
                _ => 0,
            }
        }

        //get player coords
        let mut player_x = 0;
        let mut player_y = 0;
        for (position, _player) in (&positions, &player_tag).join() {
            player_x = position.x;
            player_y = position.y;
        }  
        
        //draw all non actors first
        for (position, renderable, _actor, _player) in (&positions, &renderables, !&actors, !&player_tag).join() {
            let idx = position.x as usize + position.y as usize * visibility_map.width;

            if visibility_map.visible_tiles[idx] {
                let screen_x = {(self.screen_size.0 as i32 / 2) + (position.x - player_x) + self.horiz_offset};
                let screen_y = { (self.screen_size.1 as i32 /2) + (position.y - player_y) + self.vert_offset};

                let fg = renderable.get_shaded_foreground();
                let bg = renderable.get_shaded_background();
                self.draw_batch.set(Point::new(screen_x, screen_y), ColorPair::new(fg, bg), renderable.glyph);

            } else if visibility_map.discovered_tiles[idx] {
                
                let screen_x = {(self.screen_size.0 as i32 / 2) + (position.x - player_x) + self.horiz_offset};
                let screen_y = { (self.screen_size.1 as i32 /2) + (position.y - player_y) + self.vert_offset};

                let mut fg : RGB = (renderable.fg * RGB::from_f32(0.25, 0.25, 0.25)).to_greyscale();
                if fg.to_hsv().v < 0.25 {
                    fg = HSV::from_f32(fg.to_hsv().h, fg.to_hsv().s, fg.to_hsv().v + 0.3).to_rgb();
                }
                let bg : RGB= (renderable.bg * RGB::from_f32(0.25, 0.25, 0.25)).to_greyscale();
                self.draw_batch.set(Point::new(screen_x, screen_y), ColorPair::new(fg, bg), renderable.glyph);
            }

        }
        //draw non player actors
        for (position, renderable, _player) in (&positions, &renderables, !&player_tag).join() {
            let idx = position.x as usize + position.y as usize * visibility_map.width;
            if visibility_map.visible_tiles[idx] {
                let screen_x = {(self.screen_size.0 as i32 / 2) + (position.x - player_x) + self.horiz_offset};
                let screen_y = { (self.screen_size.1 as i32 /2) + (position.y - player_y) + self.vert_offset};
                
                let fg = renderable.get_shaded_foreground();
                let bg = renderable.get_shaded_background();
                self.draw_batch.set(Point::new(screen_x, screen_y), ColorPair::new(fg, bg), renderable.glyph);
            }
        }

        //draw player
        for (position, _player, renderable) in (&positions, &player_tag, &renderables).join() {
            let screen_x = {(self.screen_size.0 as i32 / 2) + (position.x - player_x) + self.horiz_offset};
            let screen_y = { (self.screen_size.1 as i32 /2) + (position.y - player_y) + self.vert_offset};
            
            let fg = renderable.get_shaded_foreground();
            let bg = renderable.get_shaded_background();
            self.draw_batch.set(Point::new(screen_x, screen_y), ColorPair::new(fg, bg), renderable.glyph);
        }

        // //draw inventory (temporary)
        // for (_player, invent) in (&player_tag, &inventory).join() {
        //     self.draw_batch.print(Point::new(0, 1), "Inventory:123456789");
        //     self.draw_batch.print(Point::new(0,2), format!("Gold: ${:.2}", invent.money));
        //     if invent.get_size() < 1 {
        //         self.draw_batch.print(Point::new(0, 3), "Empty :(");
        //     } else {
        //         let mut i = 0;
        //         for (item_id, amt) in invent.items.iter() {
        //             let item_name = RAW.lock().unwrap().get_item_name(*item_id);
        //             self.draw_batch.print(Point::new(0, 3 + i), format!("{} x{}", item_name, amt));
        //             i += 1;
        //         }
        //     }
        // }

        let draw_result = self.draw_batch.submit(0);
        match draw_result {
            Ok(_v) => {},
            Err(e) => { 
                error!("Error submitting batch : {}", e);
            },
        }    
    }
}

pub struct GUIRenderSystem {
    pub draw_batch : Reusable<'static, DrawBatch>,
    pub screen_size : (u32, u32),
}

impl GUIRenderSystem {
    pub fn new (draw_batch : Reusable<'static, DrawBatch>, screen_size : (u32, u32)) -> Self {
        GUIRenderSystem {
            draw_batch : draw_batch,
            screen_size : screen_size,
        }
    }

    pub fn draw_bar_horizontal (&mut self, position : (i32,i32), width : i32, current : i32, max : i32, fg : RGB, bg : RGB, with_decoration : bool) {
        let percent = current as f32 / max as f32;
        let fill_width = (percent * width as f32) as i32;

        let fill_width_float : f32 = percent * width as f32;
        let mut offset = 0;
        if with_decoration {
            offset = 1;
            self.draw_batch.set(Point::from((position.0,position.1)), ColorPair::new(RGB::named(bracket_lib::prelude::WHITE), RGB::named(bracket_lib::prelude::BLACK)), 180);
        }

        for x in 0..(width + offset) {
            let difference = fill_width_float - x as f32;
            if (x == 0 && x < fill_width) || (x > 0 && x <= fill_width) || (x == 0 && difference > 0.0) {
                if difference >= 1.0 { // 100%
                    self.draw_batch.set(Point::from((position.0 + x + offset, position.1)), ColorPair::new(fg, bg), 219);
                } else if difference < 1.00 && difference > 0.50 { // <100%
                    self.draw_batch.set(Point::from((position.0 + x + offset, position.1)), ColorPair::new(fg, bg), 178);
                } else if difference <= 0.50 && difference > 0.25 { // <50%
                    self.draw_batch.set(Point::from((position.0 + x + offset, position.1)), ColorPair::new(fg, bg), 177);
                } else if difference <= 0.25 && difference > 0.0 { // <25%
                    self.draw_batch.set(Point::from((position.0 + x + offset, position.1)), ColorPair::new(fg, bg), 176);
                }
            } else {
               self.draw_batch.set(Point::from((position.0 + x + offset, position.1)), ColorPair::new(fg, bg), 249);
            }
        }

        if with_decoration {
            self.draw_batch.set(Point::from((position.0 + (width + offset), position.1)), ColorPair::new(RGB::named(bracket_lib::prelude::WHITE), bg), 195);
        }

    }
}

impl <'a> System<'a> for GUIRenderSystem {
    type SystemData = (
        ReadStorage <'a, PlayerTag>,
        ReadStorage <'a, Actor>,
        ReadStorage <'a, PlayerCard>,
        ReadStorage <'a, Panel>
    );

    fn run (&mut self, (player_tag, actors, player_card, panels) : Self::SystemData) {
        use specs::Join;
        self.draw_batch.target(0);

        for (card, panel) in (&player_card, &panels).join() {
            let mut enabled = true;
            let mut x_coord = 0;
            match card.justification {
                Justification::RIGHT => x_coord = self.screen_size.0 - (panel.width()+1) as u32,
                Justification::LEFT => x_coord = 0,
                _ => enabled = false,
            }
            if enabled {
                self.draw_batch.draw_double_box(Rect::with_size(x_coord, 0, panel.width() as u32, self.screen_size.1-1), ColorPair::new(RGB::from_u8(255, 255, 255), RGB::from_u8(50, 50, 50)));
                //draw player stats (temporary)
                for (_player, player_actor) in (&player_tag, &actors).join() {
                    self.draw_batch.print(Point::new(x_coord+1, 1), format!("STR:{}", player_actor.strength));
                    self.draw_batch.print(Point::new(x_coord+1, 2), format!("DEX:{}", player_actor.dexterity));
                    self.draw_batch.print(Point::new(x_coord+1, 3), format!("CON:{}", player_actor.constitution));
                    self.draw_batch.print(Point::new(x_coord+1, 4), format!("WIS:{}", player_actor.wisdom));
                    self.draw_batch.print(Point::new(x_coord+1, 5), format!("INT:{}", player_actor.intelligence));
                    let hp_info = format!("HP:{}/{}", player_actor.current_health, player_actor.max_health);
                    self.draw_bar_horizontal((0,0), 10, player_actor.current_health, player_actor.max_health, RGB::from_u8(255, 0, 0), RGB::from_u8(0, 0, 0) , true);
                    self.draw_batch.print(Point::new(x_coord+1, 6), hp_info.clone());       
                }
            }
            
        }

        let draw_result = self.draw_batch.submit(1);
        match draw_result {
            Ok(_v) => {},
            Err(e) => { 
                error!("Error submitting batch : {}", e);
            },
        }
    }
}