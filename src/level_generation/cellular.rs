use rand::{Rng, StdRng};
use super::map::TileType;

const INITIAL_FILL_PERCENT : i32 = 35;
const CLEANUP_THRESHOLD : usize = 11;

#[derive(Clone)]
pub struct Region {
	pub id: String,
	pub tiles : Vec<(i32, i32)>,
	pub edge_tiles : Vec<(i32, i32)>,
	pub connected_regions : Vec<String>,
	pub size: usize,
	pub is_main_region: bool,
	pub is_connected_to_main_region: bool,
}

impl Region {
	pub fn is_tile_in_region (&mut self, other_tile : &(i32, i32)) -> bool {
		let mut is_in_region = false;
		for tile in self.tiles.clone() {
			if tile.0 == other_tile.0 && tile.1 == other_tile.1 {
				is_in_region = true;
				break;
			}
		}
		return is_in_region;
	}

	pub fn is_connected (&self, other_id: &String) -> bool {
		return self.connected_regions.contains(other_id);
	}
}

//* random fill
//* 5 generations
//* clean up walls
//* clean up floors
//* smooth

pub fn generate (width: usize, height : usize, tiles : &mut Vec<TileType>, transparency_map : &mut Vec<f32>, rng: &mut StdRng) {

	random_fill(tiles, rng);
	add_edges(width, height, tiles);
    
    for _i in 0..5 {
        perform_generation(width, height, tiles);
    }

	clean_up(width, height, tiles, TileType::Wall, CLEANUP_THRESHOLD);
	clean_up(width, height, tiles, TileType::Floor, CLEANUP_THRESHOLD);
	smooth(width, height, tiles);

	if get_all_regions(width, height, TileType::Floor, tiles).len() > 1 {
		if connect_floor_regions(width, height, tiles) == false {
			warn!("Rejected level; generating again");
			generate(width, height, tiles, transparency_map, rng);
		} else {
			smooth(width, height, tiles);
			clean_up(width, height, tiles, TileType::Wall, CLEANUP_THRESHOLD);
			add_edges(width, height, tiles);
			// cellular_automata::water(TileType::ShallowWater, &mut self.tiles, &mut self.number_generator);
			// cellular_automata::plant(&mut self.tiles, &mut self.number_generator, TileType::ShortGrass(0), 25);
			// cellular_automata::grow(&mut self.tiles, TileType::ShortGrass(0));
			// cellular_automata::remove_unseen_walls(&mut self.tiles);
		}
	}

    //after everything is done, make the transparency map.
    for i in 0..tiles.len() {
        match tiles[i] {
            TileType::Empty => transparency_map[i] = 0.0,
            TileType::Wall => transparency_map[i] = 1.0,
            TileType::Floor => transparency_map[i] = 0.0,
        }
    }
}

fn random_fill (tiles : &mut Vec<TileType>, rng: &mut StdRng) {
    info!("Seeding level...");

    for i in 0..tiles.len() {
        if rng.gen_range(0, 100) < INITIAL_FILL_PERCENT {
			tiles[i] = TileType::Wall;
		} else {
			tiles[i] = TileType::Floor;
		}
    }

}

/// Performs a single generation of the cellular automata 
/// according to the B5678/S45678 rule.
pub fn perform_generation (width : usize, height : usize, tiles : &mut Vec<TileType>) {
	info!("Digging...");

	for x in 1..width-1 {
		for y in 1..height-1 {
			let surrounding_wall_count = get_surrounding_wall_count(width, tiles, x, y);

			match tiles[x+y*width] {
				TileType::Wall => {
					if surrounding_wall_count >= 4 {
						tiles[x+y*width] = TileType::Wall;
					}
					if surrounding_wall_count < 2 {
						tiles[x+y*width] = TileType::Floor;
					}
				},
				_ => {
					if surrounding_wall_count >= 5 {
						tiles[x+y*width] = TileType::Wall;
					}
				}
			}
		}
	}
}

///Removes wall or floor regions that don't meet size requirements.
pub fn clean_up (width : usize, height : usize, tiles : &mut Vec<TileType>, t : TileType, size_threshold: usize) {
	info!("Cleaning...");
	let mut wall_regions = get_all_regions(width, height, t, tiles);

	let mut excluded_wall_regions : Vec<Region> = Vec::new();

	//exclude bad wall regions
	for i in (0..wall_regions.len()).rev() {
		if wall_regions[i].tiles.len() <= size_threshold {
			excluded_wall_regions.push(wall_regions.remove(i));
		}
	}
	let region_type = match t {
		TileType::Wall => {"wall"},
		TileType::Floor => {"floor"},
		_ => {
			error!("Invalid clean up: {:?}", t);
			"oops"
		}
	};
	info!("{} excluded {} regions;", excluded_wall_regions.len(), region_type);

	for region in excluded_wall_regions {
		for tile in region.tiles {
			let x = tile.0 as usize;
			let y = tile.1 as usize;

			tiles[x+y*width] = TileType::Floor;
		}
	}
}

/// Applies a different cellular ruleset to smooth out jagged edges.
pub fn smooth (width : usize, height : usize, tiles : &mut Vec<TileType>) {
	info!("Smoothing...");

	for x in 1..width-1 {
		for y in 1..height-1 {
			let surrounding_wall_count = get_surrounding_wall_count(width, tiles, x, y);

			if surrounding_wall_count > 4 {
				tiles[x+y*width] = TileType::Wall;
			} else if surrounding_wall_count < 4 {
				tiles[x+y*width] = TileType::Floor;
			}
		}
	}
}

pub fn add_edges (width : usize, height : usize, tiles : &mut Vec<TileType>) {
	for x in 0..width {
		if tiles [x + 0 * width] != TileType::Wall {
			tiles [x + 0 * width] = TileType::Wall;
		}
		if tiles [x + (height - 1) * width] != TileType::Wall {
			tiles[x + (height - 1) * width] = TileType::Wall;
		}
	}

	for y in 0..height {
		if tiles [0 + y * width] != TileType::Wall {
			tiles [0 + y * width] = TileType::Wall;
		}
		if tiles [(width - 1) +y * width] != TileType::Wall {
			tiles[(width -1) + y * width] = TileType::Wall;
		}
	}
}

/// Attempts to connect separated floor regions. Returns true if successful, returns false
/// if the paths calculated are too long.
pub fn connect_floor_regions (width : usize, height : usize, tiles: &mut Vec<TileType>) -> bool {
	info!("Connecting regions...");

	use bracket_lib::prelude::Bresenham;
	use bracket_lib::prelude::Point;

	let mut regions = get_all_regions(width, height, TileType::Floor, tiles);
	regions.sort_by(|a,b| b.size.cmp(&a.size));

	regions[0].is_main_region = true;
	regions[0].is_connected_to_main_region = true;

	let mut region_list_a = regions.clone();
	let mut region_list_b = regions.clone();

	let mut best_distance = 0;
	let mut possible_connection_found : bool;
	let mut best_tile_a : (i32, i32) = (0,0);
	let mut best_tile_b : (i32, i32) = (0,0);
	let mut best_region_a = String::new();
	let mut best_region_b = String::new();

	let mut look_for_connection_to_main : bool;

	for region_a in &mut region_list_a {
		possible_connection_found = false;
		look_for_connection_to_main = !region_a.is_connected_to_main_region;

		for region_b in &mut region_list_b {

			// Conditions to skip iteration of unwanted regions.
			// if the two regions are the same.
			if region_a.id == region_b.id {
				continue;
			//if the region we are looking at isn't connected to the main region but we are looking for a connection.
			} else if look_for_connection_to_main && !region_b.is_connected_to_main_region {
				continue;
			// if the region we are looking at is already connected.
			} else if region_a.is_connected(&region_b.id.clone()) {
				continue;
			}

			for tile_a in &region_a.edge_tiles {
				for tile_b in &region_b.edge_tiles {
					
					let distance = (tile_a.0 as i32 - tile_b.0 as i32).pow(2) + (tile_a.1 as i32 - tile_b.1 as i32).pow(2);
					
					if distance < best_distance || !possible_connection_found {
						best_distance = distance;
						possible_connection_found = true;
						best_tile_a = (tile_a.0 as i32, tile_a.1 as i32);
						best_tile_b = (tile_b.0 as i32, tile_b.1 as i32);
						best_region_a = region_a.id.clone();
						best_region_b = region_b.id.clone();
					} 
				}
			}
		}

		region_a.connected_regions.push(best_region_b.clone());

		for region_b in &mut region_list_b {
			if region_b.id == best_region_b {
				region_b.connected_regions.push(best_region_a.clone());
				if region_a.is_connected_to_main_region {
					region_b.is_connected_to_main_region = true;
				} else if region_b.is_connected_to_main_region {
					region_a.is_connected_to_main_region = true;
				}
			}
		}

		let mut path = Bresenham::new(Point::from_tuple(best_tile_a), Point::from_tuple(best_tile_b));

		let path_length = path.count() as i32;

		path = Bresenham::new(Point::from_tuple(best_tile_a), Point::from_tuple(best_tile_b));

		let mut dig_radius = path_length / 6;
		
		// A large dig_radius looks unnatural. Generate a new map if there are any paths that require a large dig radius.
		if dig_radius >= 4 {
			return false;
		}

		if dig_radius < 1 {
			dig_radius = 1;
		}

		info!("Digging paths...");
		for point in path {
			dig_circle(width, height, point.to_tuple(), dig_radius, tiles);
		}
	}
	return true;
}

/// Digs a circle of a given radius of floors at a point 
pub fn dig_circle (width : usize, height : usize, tile : (i32, i32), radius: i32, tiles: &mut Vec<TileType>) {

	for x in -radius..radius {
		for y in -radius..radius {
			if x*x + y*y <= radius*radius {
				let dig_x = tile.0 + x;
				let dig_y = tile.1 + y;
				if (dig_x >= 0 && dig_x < width as i32) && (dig_y >= 0 && dig_y < height as i32) {
					let idx = dig_x as usize + dig_y as usize * width;
					tiles[idx] = TileType::Floor;
				} 
			}
		}
	}
}

// Returns a `Vec` of all regions in the map of a given `TileType`.
pub fn get_all_regions (width : usize, height : usize, tile_type: TileType, tiles: &mut Vec<TileType>) -> Vec<Region> {

	let mut regions: Vec<Region> = Vec::new();
	let mut map_flags: Vec<u8> = vec![0; width*height];

	for x in 1..width {
		for y in 1..height {
			if (map_flags[x+y*width] == 0) && (tiles[x+y*width] == tile_type){
				let new_region = get_region(width, height, (x,y), tiles);

				for tile in new_region.tiles.iter(){
					let idx = tile.0 as usize + tile.1 as usize * width;
					map_flags[idx] = 1;
				}

				regions.push(new_region);
			}
		}
	}
	return regions;
}

/// Gets a region using flood-fill. The type of region is the same as the `TileType` of the starting tile.
pub fn get_region (width : usize, height: usize, start_tile: (usize, usize), tiles : &mut Vec<TileType>) -> Region{
	use std::collections::VecDeque;
	use crate::state::time::get_current_time_millis;

	let mut region_tiles: Vec<(i32,i32)> = Vec::new();
	let mut map_flags : Vec<u8> = vec![0; width*height];

	let current_tile_type = tiles[start_tile.0 + start_tile.1 * width].clone();

	let mut queue : VecDeque<(i32,i32)> = VecDeque::new();

	queue.push_back((start_tile.0 as i32, start_tile.1 as i32));
	map_flags[start_tile.0 + start_tile.1 * width] = 1;
	while !queue.is_empty() {

		let tile = queue.pop_front().unwrap();

		region_tiles.push(tile);
		for dx in (tile.0 - 1)..(tile.0 + 2) {
			for dy in (tile.1 - 1)..(tile.1 + 2) {
				if ((dx >= 0 && dx < width as i32) && (dy >= 0 && dy < height as i32)) && (dy == tile.1 || dx == tile.0){
					let idx = dx as usize + dy as usize * width;
					if (map_flags[idx] == 0) && (tiles[idx] == current_tile_type) {
						map_flags[idx] = 1;
						queue.push_back((dx, dy));
					}
				}
			}
		}
	}
	let size = region_tiles.len();
	let mut edge_tiles = region_tiles.clone();
	edge_tiles.retain(|a| is_edge_tile(width, height, a.0 as i32, a.1 as i32, tiles));

	let region = Region {
		id: format!("{}", get_current_time_millis()),
		tiles: region_tiles,
		edge_tiles: edge_tiles,
		connected_regions : Vec::new(),
		size: size,
		is_main_region: false,
		is_connected_to_main_region: false,
	};
	return region;
}

/// Returns ture of the given coordates are of a tile on the edge of a region.
pub fn is_edge_tile (width: usize, height : usize, x: i32, y: i32, tiles : &mut Vec<TileType>) -> bool {

	for dx in (x - 1)..(x + 2) {
		for dy in (y-1)..(y+2) {
			if (dx != x) || (dy != y) {
				if (dx < 0 || dx >= width as i32) || (dy < 0 || dy >= height as i32) {
					return true;
					
				} else {
					let idx = dx as usize + dy as usize * width;
					return tiles[idx] != tiles[idx];
				}
			}
		}
	}
	return false;
}

/// Returns the number of walls out of the 8 neighbors of a given tile.
fn get_surrounding_wall_count (width : usize, tiles : &mut Vec<TileType>, x: usize, y: usize) -> i32 {
	let mut wallcount = 0;
	for dx in (x - 1)..(x + 2) {
		for dy in (y - 1)..(y + 2) {
			if !((dx == x) && (dy == y)) {
				match tiles[dx + dy * width] {
					TileType::Wall => wallcount += 1,
					_ => {},
				}
			}
		}
	}
	return wallcount;
}
