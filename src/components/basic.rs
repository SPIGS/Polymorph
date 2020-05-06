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
    pub shading : (f32, f32, f32),
}

impl Renderable {
    pub fn new (glyph: u16, foreground_color : RGB, background_color : RGB, shadeless : bool) -> Self {
        Renderable {
            glyph : glyph,
            fg : foreground_color,
            bg : background_color,
            shadeless : shadeless,
            shading : (1.0, 1.0, 1.0),
        }
    }

    pub fn new_from_char(glyph : char, foreground_color : RGB, background_color : RGB, shadeless : bool) -> Self{
        Renderable {
            glyph : glyph as u16,
            fg : foreground_color,
            bg : background_color,
            shadeless : shadeless,
            shading : (1.0, 1.0, 1.0),
        }
    }

    pub fn get_shaded_foreground (&self) -> RGB {
        let obj_r = self.fg.r;
        let obj_g = self.fg.g;
        let obj_b = self.fg.b;

        return RGB::from_f32(obj_r * self.shading.0, obj_g * self.shading.1, obj_b * self.shading.2)
    }

    pub fn get_shaded_background (&self) -> RGB {
        let obj_r = self.bg.r;
        let obj_g = self.bg.g;
        let obj_b = self.bg.b;

        return RGB::from_f32(obj_r * self.shading.0, obj_g * self.shading.1, obj_b * self.shading.2)
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
}

impl Light {
    pub fn new (radius : u32, intensity : f32) -> Self {
        Light {
            radius : radius,
            intensity : intensity,
        }
    }
}