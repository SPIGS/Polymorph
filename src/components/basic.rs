use specs::{Component, VecStorage, DenseVecStorage};
use bracket_lib::prelude::RGB;

use std::collections::HashMap;

use crate::raw::ItemRaw;
use crate::raw::{RAW};

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

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Renderable {
    pub glyph: u16,
    pub fg : RGB,
    pub bg : RGB,
    pub shadeless : bool,
    pub shading : RGB,
}

impl Renderable {
    pub fn new (glyph: u16, foreground_color : RGB, background_color : RGB, shadeless : bool) -> Self {
        Renderable {
            glyph : glyph,
            fg : foreground_color,
            bg : background_color,
            shadeless : shadeless,
            shading : RGB::from_f32(0.0, 0.0, 0.0),
        }
    }

    pub fn new_from_char(glyph : char, foreground_color : RGB, background_color : RGB, shadeless : bool) -> Self{
        Renderable {
            glyph : glyph as u16,
            fg : foreground_color,
            bg : background_color,
            shadeless : shadeless,
            shading : RGB::from_f32(0.0, 0.0, 0.0),
        }
    }

    pub fn get_shaded_foreground (&self) -> RGB {
        let combined : RGB;

        // if the light or obj has no hue
        if (self.shading.r == self.shading.g && self.shading.g == self.shading.b) || (self.fg.r == self.fg.g && self.fg.g == self.fg.b){
            combined = self.fg * self.shading;
        } else {
            let value = self.shading.to_hsv().v;
            combined = RGB::from_f32((self.fg.r + self.shading.r) * value, (self.fg.g + self.shading.g) * value, (self.fg.b + self.shading.b) * value);
        }
        
        return combined;
    }

    pub fn get_shaded_background (&self) -> RGB {
        return (self.bg + self.shading) * 0.5;
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
    pub radius : u32,
    pub intensity : f32,
    pub color : RGB,
}

impl Light {
    pub fn new (radius : u32, intensity : f32, color : RGB) -> Self {
        Light {
            radius : radius,
            intensity : intensity,
            color : color,
        }
    }
}