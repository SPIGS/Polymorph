use super::map::tile::*;
use super::cellular::Region;
use rand::{Rng, StdRng};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FeatureType {
    CavernFeatures,
    NoFeatures,
}

pub fn make_spider_nest (width : usize, tiles : &mut Vec<TileType>, region : Region, rng : &mut StdRng) {
    debug!("Placing nest...");
    //place thin webs
    for edge_tile in region.edge_tiles.clone() {
        let chance = rng.gen_range(0,100);
        if chance < 40 {
            let idx = (edge_tile.0 + edge_tile.1 * width as i32) as usize;
            if tiles[idx] != TileType::Wall {
                tiles[idx] = TileType::ThinWebs;
            }
        }
    }

    //place thick webs
    for tile in region.tiles.clone() {
        let chance = rng.gen_range(0,100);
        if chance < 20 {
            let idx = (tile.0 + tile.1 * width as i32) as usize;
            if tiles[idx] != TileType::Wall {
                tiles[idx] = TileType::ThickWebs;
            }
        }
    }

    //place egg sacks
    let mut amount_remaining = rng.gen_range(1,5);
    for tile in region.tiles.clone() {
        let idx = (tile.0 + tile.1 * width as i32) as usize;
        if tiles[idx] == TileType::Floor {
            
            tiles[idx] = TileType::EggSac;
            
            amount_remaining -= 1;
            if amount_remaining <= 0 {
                break;
            }
        }
    }
}
pub fn make_camp (width : usize, tiles : &mut Vec<TileType>, region : Region) {
    debug!("Placing camp...");
    //TODO add check for size of region so multiple camps can be added
    let mut center = (0,0);
    let mut area_found = false;
    for tile in region.tiles {
        center = (tile.0, tile.1);
        area_found = true;

        for x in tile.0-4..tile.0+5 {
            for y in tile.1-4..tile.1+5 {
                let mut distance = (x-tile.0).pow(2) + (y-tile.1).pow(2);
                distance = (distance as f64).sqrt() as i32;

                if distance <= 4 {
                    let idx = x+y*width as i32;
                    if idx  as usize >= tiles.len() || idx < 0 {
                        area_found = false;
                        break;
                    } else {
                        if tiles[idx as usize] == TileType::Wall || tiles[idx as usize] == TileType::Empty {
                            area_found = false;
                            break;
                        }
                    }
                }
            }

            if !area_found {
                break;
            }
        }
        if area_found {
            break;
        }
    }
    if area_found {
        for x in center.0-4..center.0+5 {
            for y in center.1-4..center.1+5 {
                let mut distance = (x-center.0).pow(2) + (y-center.1).pow(2);
                distance = (distance as f64).sqrt() as i32;

                if distance <= 4 {
                    let idx = (x+y*width as i32) as usize;
                    tiles[idx] = TileType::Floor;
                }
            }
        }
        tiles[(center.0+center.1*width as i32) as usize] = TileType::Fire;

        tiles[(center.0+(center.1 - 4)*width as i32) as usize] = TileType::TentTopCenter;
        tiles[((center.0-1)+(center.1 - 4)*width as i32) as usize] = TileType::TentTopLeft;
        tiles[((center.0+1)+(center.1 - 4)*width as i32) as usize] = TileType::TentTopRight;
        tiles[(center.0+(center.1 - 3)*width as i32) as usize] = TileType::TentBottomCenter;
        tiles[((center.0-1)+(center.1 - 3)*width as i32) as usize] = TileType::TentBottomLeft;
        tiles[((center.0+1)+(center.1 - 3)*width as i32) as usize] = TileType::TentBottomRight;

        tiles[((center.0 + 2)+center.1*width as i32) as usize] = TileType::CampSeat;
        tiles[((center.0 - 2)+center.1*width as i32) as usize] = TileType::CampSeat;

    } else {
        debug!("Couldn't place camp");
    }
}