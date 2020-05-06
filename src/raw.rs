use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;

use std::collections::HashMap;

lazy_static! {
    pub static ref RAW : Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

pub struct RawMaster {
    pub raws : Raws,
    pub name_to_id : HashMap<String, u32>,
    pub id_to_name : HashMap<u32, String>,
}

impl RawMaster {
    pub fn empty () -> Self {

        RawMaster {
            raws : Raws::empty(),
            name_to_id : HashMap::new(),
            id_to_name : HashMap::new(),
        }
    }

    /// returns the id of an item given its name.
    pub fn get_item_id (&self, name : String) -> u32 {
        let result = self.name_to_id.get(&name);
        match result {
            Some(id) => {
                return *id;
            },
            None => {
                error!("No item found with the name \"{}\". Maybe the raw files were edited or the raws have not been loaded?", name);
                return 0;
            },
        }
    }

    pub fn get_item_name (&self, id : u32) -> String {
        let result = self.id_to_name.get(&id);
        match result {
            Some(name) => {
                return name.clone();
            },
            None => {
                error!("No item found with the id \"{}\". Maybe the raw files were edited or the raws have not been loaded?", id);
                return "Diamond".to_string();
            },
        }
    }

    /// Given an id returns the corresponding item.
    pub fn get_item (&self, id : u32) -> ItemRaw {
        let item = self.raws.items[id as usize].clone();
        return item;
    }
    /// loads the raws into memory
    pub fn load_raws (&mut self) {

        // attempt to read raw files; if the raw file isn't found, log an error.
        let file_result = fs::read_to_string("raws/items.json");
        let mut serialized = "".to_string();
        match file_result {
            Ok(t) => {
                serialized = t;
            },
            Err(e) => {
                error!("ERROR READING RAW FILE : {}", e);
            },
        }

        // attempt to deserialize the data from the raw files, if not log error
        let deserialization_result : Result<Raws, serde_json::Error> = serde_json::from_str(&serialized);
        match deserialization_result {
            Ok(t) => {
                self.raws.items = t.items.clone();
            },
            Err(e) => {
                error!("ERROR LOADING RAW FILE : {}", e);
            },
        }

        //index items for quick look up
        let length = self.raws.items.len();
        for i in 0..length {
            let name = self.raws.items[i].name.clone();
            self.name_to_id.insert(name.clone(), i as u32);
            self.id_to_name.insert(i as u32, name.clone());
        }
    }

}



#[derive(Deserialize, Debug, Serialize)]
pub struct Raws {
    pub items : Vec<ItemRaw>,
}

impl Raws {
    pub fn empty () -> Self {
        Raws {
            items : Vec::new()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemRaw {
    pub item_type : String,
    pub name : String,
    pub weight : f32,
    pub rarity : f32,
    pub value : f32,
    pub renderable : Option<RenderableRaw>,
    pub potion : Option<PotionRaw>,
    pub melee_weapon : Option<MeleeWeaponRaw>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderableRaw {
    pub character_code : i32,
    pub fg : String,
    pub bg : String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeleeWeaponRaw {
    pub base_damage : u32,
    pub effects : Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PotionRaw {
    pub effects : Option<Vec<String>>,
}