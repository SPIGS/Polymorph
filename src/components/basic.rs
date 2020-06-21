use specs::{Component, VecStorage, DenseVecStorage};
use bracket_lib::prelude::RGB;

use std::collections::HashMap;

use crate::systems::render::ObjectShader;

use crate::raw::ItemRaw;
use crate::raw::{RAW};

use bracket_lib::prelude::{NoiseType, Interp, FastNoise};
use bracket_lib::prelude::RandomNumberGenerator;

#[derive(Debug, PartialEq, Component)]
#[storage(DenseVecStorage)]
pub struct Position {
    pub x : i32,
    pub y : i32,
}

impl Position {
    pub fn new (x : i32, y : i32)-> Self {
        Position {
            x : x,
            y : y,
        }
    }
}

const BG_SHADE_MULT : f32 = 0.15;

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Renderable {
    pub glyph: u16,
    pub fg : RGB,
    pub bg : RGB,
    pub fg_shader : ObjectShader,
    pub bg_shader : ObjectShader,
    pub shading : RGB,
}

impl Renderable {
    pub fn new (glyph: u16, foreground_color : RGB, background_color : RGB, fg_shader : ObjectShader, bg_shader : ObjectShader) -> Self {
        Renderable {
            glyph : glyph,
            fg : foreground_color,
            bg : background_color,
            fg_shader : fg_shader,
            bg_shader : bg_shader,
            shading : RGB::from_f32(0.0, 0.0, 0.0),
        }
    }

    pub fn new_from_char(glyph : char, foreground_color : RGB, background_color : RGB, fg_shader : ObjectShader, bg_shader : ObjectShader) -> Self{
        Renderable {
            glyph : glyph as u16,
            fg : foreground_color,
            bg : background_color,
            fg_shader : fg_shader,
            bg_shader : bg_shader,
            shading : RGB::from_f32(0.0, 0.0, 0.0),
        }
    }

    pub fn get_shaded_foreground (&self) -> RGB {
        let shaded_foreground : RGB;

        if self.fg_shader == ObjectShader::Foreground {
            // if the light or obj has no hue
            if (self.shading.r == self.shading.g && self.shading.g == self.shading.b) || (self.fg.r == self.fg.g && self.fg.g == self.fg.b){
                shaded_foreground = self.fg * self.shading;
            } else {
                let value = self.shading.to_hsv().v;
                shaded_foreground = RGB::from_f32((self.fg.r + self.shading.r) * value, (self.fg.g + self.shading.g) * value, (self.fg.b + self.shading.b) * value);
            }
        } else if self.fg_shader == ObjectShader::Background {
            shaded_foreground = (self.fg + self.shading) * BG_SHADE_MULT
        } else {
            shaded_foreground = self.fg;
        }

        return shaded_foreground;
    }

    pub fn get_shaded_background (&self) -> RGB {
        let shaded_background : RGB;

       if self.bg_shader == ObjectShader::Foreground {
            // if the light or obj has no hue
            if (self.shading.r == self.shading.g && self.shading.g == self.shading.b) || (self.fg.r == self.fg.g && self.fg.g == self.fg.b){
                shaded_background = self.bg * self.shading;
            } else {
                let value = self.shading.to_hsv().v;
                shaded_background = RGB::from_f32((self.bg.r + self.shading.r) * value, (self.bg.g + self.shading.g) * value, (self.bg.b + self.shading.b) * value);
            }
        } else if self.bg_shader == ObjectShader::Background {
            shaded_background = (self.bg + self.shading) * BG_SHADE_MULT
        } else {
            shaded_background = self.bg
        }
        
        return shaded_background;
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Inventory {
    pub items : HashMap<u32, u32>,
    pub money : f32,
    size : usize,
}

impl Inventory {
    pub fn new () -> Self {
        Inventory {
            items : HashMap::new(),
            money : 0.0,
            size : 0,
        }
    }

    pub fn add_item (&mut self, item : ItemRaw) {
        let name = item.name.clone();
        let id = RAW.lock().unwrap().get_item_id(name);
        if self.items.contains_key(&id) {
            self.items.entry(id).and_modify(|amt| *amt += 1);
        } else {
            self.items.insert(id, 1);
        }
        self.size += 1;
    }
    pub fn get_size (&self) -> usize {
        return self.size;
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ItemWrapper {
   pub item_data : ItemRaw,
}

impl ItemWrapper{
    pub fn new (data : ItemRaw) -> Self {
        ItemWrapper {
            item_data : data,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Currency {
    pub amt : u32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Actor {
    pub strength : u8,
    pub dexterity : u8,
    pub constitution : u8,
    pub wisdom : u8,
    pub intelligence : u8,
    pub max_health : i32,
    pub current_health : i32,
}

impl Actor {
    pub fn new () -> Self {
        Actor {
            strength : 10,
            dexterity : 10,
            constitution : 10,
            wisdom : 10,
            intelligence : 10,
            max_health: 100,
            current_health : 100,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Light {
    pub org_rad : u32,
    pub cur_rad : u32,
    pub intensity : f32,
    pub color : RGB,
}

impl Light {
    pub fn new (radius : u32, intensity : f32, color : RGB) -> Self {
        Light {
            org_rad : radius,
            cur_rad : radius,
            intensity : intensity,
            color : color,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ColorLerp {
    pub color_a : RGB,
    pub color_b : RGB,
    pub rate : f32,
    accumulator : f32,
}

impl ColorLerp {
    pub fn new (color_a : RGB, color_b : RGB, rate : f32, offset : f32) -> Self {
        ColorLerp {
            color_a : color_a,
            color_b : color_b,
            rate : rate,
            accumulator : offset,
        }
    }

    pub fn lerp (&mut self, delta : f32) {
        self.accumulator += delta;
        if self.accumulator / self.rate >= 1.0 {
            self.accumulator = 0.0;
            let temp = self.color_a;
            self.color_a = self.color_b;
            self.color_b = temp;
        }
    }

    pub fn get_current_color (&self) -> RGB {
        return self.color_a.lerp(self.color_b, self.accumulator / self.rate);
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CycleAnimation {
    pub rate : f32,
    pub frames : Vec<u16>,
    current_frame : usize,
    accumulator : f32,
}

impl CycleAnimation {
    pub fn new (rate : f32, frames : Vec<u16>) -> Self {
        CycleAnimation {
            rate : rate,
            frames : frames,
            current_frame : 0,
            accumulator : 0.0,
        }
    }

    pub fn cycle (&mut self, delta : f32) {
        self.accumulator += delta;
        if self.accumulator / self.rate >= 1.0 {
            self.accumulator = 0.0;
            self.current_frame += 1;
            if self.current_frame == self.frames.len() {
                self.current_frame = 0;
            }
        }
    }

    pub fn get_current_frame (&self) -> u16 {
        return self.frames[self.current_frame];
    }
}

#[derive(Component)]
pub struct LightFlicker {
    accumulator : i32,
}

impl LightFlicker {
    pub fn new () -> Self {
        LightFlicker {
            accumulator : 0,
        }
    }
    
    pub fn next (&mut self) -> f32{
        
        let mut noise = FastNoise::new();
        noise.set_seed(1337);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_interp(Interp::Linear);
        
        let noise_val = noise.get_noise(self.accumulator as f32 / 160.0, self.accumulator as f32 / 100.0).abs() * 0.65;
        let percent = 1.0 - noise_val;
        
        self.accumulator += 1;
        
        if self.accumulator > 1000 {
            self.accumulator = 0;
        }

        return percent;
    }
}