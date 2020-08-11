use specs::{System, ReadStorage, Read};
use bracket_lib::prelude::DrawBatch;
use bracket_lib::prelude::Point;
use bracket_lib::prelude::ColorPair;
use bracket_lib::prelude::{RGB, HSV};
use bracket_lib::prelude::Rect;
use bracket_lib::prelude::{TextBuilder, TextBlock};
use object_pool::Reusable;

use crate::components::basic::{Position, Renderable, Inventory, Actor};
use crate::components::gui::{PlayerCard, HorizontalAlignment, VerticalAlignment, Panel, TextBox, DebugInfoBox};
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
            self.horiz_offset = match card.alignment {
                HorizontalAlignment::RIGHT => -(self.screen_size.0 as i32/8),
                HorizontalAlignment::LEFT => (self.screen_size.0 as i32/8),
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

        // for (_player, position) in (&player_tag, &positions).join() {
        //     self.draw_batch.print(Point::new(10, self.screen_size.1 /2), format!("X: {}; Y:{};", position.x, position.y));
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

    pub fn draw_panel (&mut self, panel : &Panel) {
        //get the border color
        let border_color : RGB = if panel.decorated {
                panel.decor_color
            } else {
                RGB::from_u8(0,0,0)
            };

        //draw the panel
        self.draw_batch.draw_double_box(Rect::with_size(panel.x, panel.y, panel.width, panel.height), ColorPair::new(border_color, panel.panel_color));
        
        //print title if the panel has one
        match &panel.title {
            Some(t) => {
                self.draw_batch.print_color_centered_at(Point::new(panel.horizontal_center(), panel.y), t, ColorPair::new(panel.title_color, panel.panel_color));
            },
            None => {},
        }
    }

    pub fn draw_textbox (&mut self, textbox : &TextBox, x : i32, y : i32) {
        let mut block = TextBlock::new(x, y, textbox.max_width as i32, textbox.max_height as i32);
        block.print(textbox.get_buffer());
        block.render_to_draw_batch(&mut self.draw_batch);
    }
}

impl <'a> System<'a> for GUIRenderSystem {
    type SystemData = (
        ReadStorage <'a, Panel>,
        ReadStorage<'a, TextBox>,
        ReadStorage<'a, DebugInfoBox>,
    );

    fn run (&mut self, (panels, textboxes, debug_info_box) : Self::SystemData) {
        use specs::Join;
        self.draw_batch.target(0);

        // panels
        for (panel, _debug) in (&panels, !&debug_info_box).join() {
            if panel.enabled {
                self.draw_panel(&panel);
            }
        }

        // textboxes
        for (textbx, panel, _debug) in (&textboxes, &panels, !&debug_info_box).join() {
            if panel.enabled {
                self.draw_textbox(textbx, panel.x+1, panel.y+1);
            }
        }

        //debug panel and info box
        for (panel, textbx, _debug) in (&panels, &textboxes, &debug_info_box).join() {
            self.draw_panel(&panel);
            self.draw_textbox(textbx, panel.x+1, panel.y+1);
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