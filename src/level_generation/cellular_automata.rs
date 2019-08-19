use rand::{Rng, StdRng};
use crate::level_generation::map::{TileType, Region, is_safe};

const INITIAL_FILL_PERCENT : i32 = 35;

/// Randomly fills the tile map with random floors and walls according to
/// an `initial_fill_percent` based on the map's `number_generator`.
/// 
/// Edges of the map will be walls.
pub fn random_fill (tiles : &mut Vec<Vec<TileType>>, number_generator: &mut StdRng) {
	println!("Seeding level...");
	let width = tiles.len();
	let height = tiles[0].len();

	for x in 1..width-1 {
		for y in 1..height-1 {
			if number_generator.gen_range(0, 100) < INITIAL_FILL_PERCENT {
				tiles[x][y] = TileType::Wall;
			} else {
				tiles[x][y] = TileType::Floor;
			}
		}
	}
}

/// Performs a single generation of the cellular automata 
/// according to the B5678/S45678 rule.
pub fn perform_generation (tiles : &mut Vec<Vec<TileType>>) {
	println!("Digging...");
	let width = tiles.len();
	let height = tiles[0].len();

	for x in 1..width-1 {
		for y in 1..height-1 {
			let surrounding_wall_count = get_surrounding_wall_count(tiles, x, y);

			match tiles[x][y] {
				TileType::Wall => {
					if surrounding_wall_count >= 4 {
						tiles[x][y] = TileType::Wall;
					}
					if surrounding_wall_count < 2 {
						tiles[x][y] = TileType::Floor;
					}
				},
				_ => {
					if surrounding_wall_count >= 5 {
						tiles[x][y] = TileType::Wall;
					}
				}
			}
		}
	}
}

/// Applies a different cellular ruleset to smooth out jagged edges.
pub fn smooth (tiles : &mut Vec<Vec<TileType>>) {
	println!("Smoothing...");

	let width = tiles.len();
	let height = tiles[0].len();

	for x in 1..width-1 {
		for y in 1..height-1 {
			let surrounding_wall_count = get_surrounding_wall_count(tiles, x, y);

			if surrounding_wall_count > 4 {
				tiles[x][y] = TileType::Wall;
			} else if surrounding_wall_count < 4 {
				tiles[x][y] = TileType::Floor;
			}
		}
	}
}

/// Returns the number of walls out of the 8 neighbors of a given tile.
fn get_surrounding_wall_count (tiles : &mut Vec<Vec<TileType>>, x_coord: usize, y_coord: usize) -> i32 {
	let mut wallcount = 0;
	for x in (x_coord - 1)..(x_coord + 2) {
		for y in (y_coord - 1)..(y_coord + 2) {
			if !((x == x_coord) && (y == y_coord)) {
				match tiles[x][y] {
					TileType::Wall => wallcount += 1,
					_ => {},
				}
			}
		}
	}
	return wallcount;
}

///Removes wall regions that don't meet size requirements.
pub fn clean_up_walls (tiles : &mut Vec<Vec<TileType>>, size_threshold: usize) {
	println!("Cleaning...");
	let mut wall_regions = get_all_regions(TileType::Wall, tiles);

	let mut excluded_wall_regions : Vec<Region> = Vec::new();

	//exclude bad wall regions
	for i in (0..wall_regions.len()).rev() {
		if wall_regions[i].tiles.len() <= size_threshold {
			excluded_wall_regions.push(wall_regions.remove(i));
		}
	}

	println!("{} excluded wall regions;", excluded_wall_regions.len());

	for region in excluded_wall_regions {
		for tile in region.tiles {
			let x = tile.0;
			let y = tile.1;

			tiles[x][y] = TileType::Floor;
		}
	}
}

/// Removes floor regions that don't meet size requirements.
pub fn clean_up_floors (tiles : &mut Vec<Vec<TileType>>) {
	println!("Cleaning...");

	let mut floor_regions = get_all_regions(TileType::Floor, tiles);
	let floor_threshold_size = 200;

	let mut excluded_floor_regions : Vec<Region> = Vec::new();

	//exclude bad floor regions
	for i in (0..floor_regions.len()).rev() {
		if floor_regions[i].tiles.len() <= floor_threshold_size {
			excluded_floor_regions.push(floor_regions.remove(i));
		}
	}

	println!("{} excluded floor regions;", excluded_floor_regions.len());

	//change bad floors to walls
	for region in excluded_floor_regions {
		for tile in region.tiles {
			let x = tile.0;
			let y = tile.1;
			tiles[x][y] = TileType::Wall;
		}
	}
}
// Returns a `Vec` of all regions in the map of a given `TileType`.
pub fn get_all_regions (tile_type: TileType, tiles: &mut Vec<Vec<TileType>>) -> Vec<Region> {
	let width = tiles.len();
	let height = tiles[0].len();

	let mut regions: Vec<Region> = Vec::new();
	let mut map_flags: Vec<Vec<u8>> = vec![vec![0; height]; width];

	for x in 1..width {
		for y in 1..height {
			if (map_flags[x][y] == 0) && (tiles[x][y] == tile_type){
				let new_region = get_region((x,y), tiles);

				for tile in new_region.tiles.iter(){
					map_flags[tile.0][tile.1] = 1;
				}

				regions.push(new_region);
			}
		}
	}
	return regions;
}

/// Gets a region using flood-fill. The type of region is the same as the `TileType` of the starting tile.
pub fn get_region (start_tile: (usize, usize), tiles : &mut Vec<Vec<TileType>>) -> Region{
	use std::collections::VecDeque;
	use crate::application::get_current_time_millis;

	let width = tiles.len() as i32;
	let height = tiles[0].len() as i32;

	let mut region_tiles: Vec<(usize,usize)> = Vec::new();
	let mut map_flags : Vec<Vec<u8>> = vec![vec![0; height as usize]; width as usize];

	let current_tile_type = tiles[start_tile.0][start_tile.1].clone();

	let mut queue : VecDeque<(usize,usize)> = VecDeque::new();

	queue.push_back(start_tile);
	map_flags[start_tile.0][start_tile.1] = 1;
	while !queue.is_empty() {

		let tile = queue.pop_front().unwrap();

		region_tiles.push(tile);
		for x in (tile.0 as i32 - 1)..(tile.0 as i32 + 2) {
			for y in (tile.1 as i32 - 1)..(tile.1 as i32 + 2) {
				if ((x >= 0 && x < width) && (y >= 0 && y < height)) && (y == tile.1 as i32 || x == tile.0 as i32){
					if (map_flags[x as usize][y as usize] == 0) && (tiles[x as usize][y as usize] == current_tile_type) {
						map_flags[x as usize][y as usize] = 1;
						queue.push_back((x as usize, y as usize));
					}
				}
			}
		}
	}
	let size = region_tiles.len();
	let mut edge_tiles = region_tiles.clone();
	edge_tiles.retain(|a| is_edge_tile(a.0 as i32, a.1 as i32, tiles));

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
pub fn is_edge_tile (x_coord: i32, y_coord : i32, tiles : &mut Vec<Vec<TileType>>) -> bool {

	let width = tiles.len() as i32;
	let height = tiles[0].len() as i32;

	for x in (x_coord-1)..(x_coord+2) {
		for y in (y_coord-1)..(y_coord+2) {
			if (x != x_coord) || (y != y_coord) {
				if (x < 0 || x >= width) || (y < 0 || y >= height) {
					return true;
					
				} else {
					return tiles[x as usize][y as usize] != tiles[x_coord as usize][y as usize];
				}
			}
		}
	}
	return false;
}

/// Attempts to connect separated floor regions. Returns true if successful, returns false
/// if the paths calculated are too long.
pub fn connect_floor_regions (tiles: &mut Vec<Vec<TileType>>) -> bool {
	println!("Connecting regions...");

	use tcod::line::Line;

	let mut regions = get_all_regions(TileType::Floor, tiles);
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

		let mut path = Line::new(best_tile_a, best_tile_b);

		let path_length = path.count() as i32;

		path = Line::new(best_tile_a, best_tile_b);

		let mut dig_radius = path_length / 6;
		
		// A large dig_radius looks unnatural. Generate a new map if there are any paths that require a large dig radius.
		if dig_radius >= 4 {
			return false;
		}

		if dig_radius < 1 {
			dig_radius = 1;
		}

		println!("Digging paths...");
		for point in path {
			dig_circle(point, dig_radius, tiles);
		}
	}
	return true;
}

/// Digs a circle of a given radius of floors at a point 
pub fn dig_circle (tile : (i32, i32), radius: i32, tiles: &mut Vec<Vec<TileType>>) {
	let width = tiles.len() as i32;
	let height = tiles[0].len() as i32;

	for x in -radius..radius {
		for y in -radius..radius {
			if x*x + y*y <= radius*radius {
				let dig_x = tile.0 + x;
				let dig_y = tile.1 + y;
				if dig_x > 0 && dig_x < width && dig_y > 0 && dig_y < height {
					tiles[dig_x as usize][dig_y as usize] = TileType::Floor;
				} 
			}
		}
	}
}

/// Like `random_fill` but places a type of foliage on `TileType::Floor` according to a given density.
pub fn plant (tiles: &mut Vec<Vec<TileType>>, number_generator: &mut StdRng, flora_type: TileType, density: i32) {
	println!("Planting...");

	let width = tiles.len();
	let height = tiles[0].len();

	for x in 1..width-1 {
		for y in 1..height-1 {
			if tiles[x][y] == TileType::Floor {
				let chance = number_generator.gen_range(0, 100);
				if chance < density {
					if flora_type == TileType::ShortGrass(0) {
						tiles[x as usize][y as usize] = TileType::ShortGrass(0);
					} else if flora_type == TileType::SmallMushroom {
						tiles[x as usize][y as usize] = TileType::SmallMushroom;
					}
				}
			}
		}
	}
}

/// Performs a cellular generation on foilage. And calculates how far each grass tile is from water.
pub fn grow (tiles: &mut Vec<Vec<TileType>>, flora_type: TileType, ) {
	println!("Growing...");

	let width = tiles.len();
	let height = tiles[0].len();

	for x in 1..width-1 {
		for y in 1..height-1 {
			let amount_of_flora = get_amount_of_flora(x,y, flora_type, &tiles);
			//Grass
			if (flora_type == TileType::ShortGrass(0)) || (flora_type == TileType::TallGrass(0)){
				let amount_of_water = get_amount_of_flora(x, y, TileType::ShallowWater, &tiles);
				if tiles[x][y] == TileType::ShortGrass(0) {	
					if amount_of_flora.0 + amount_of_flora.1 >= 3 {
						tiles[x][y] = TileType::TallGrass(0);
					}
					if amount_of_flora.0 + amount_of_flora.1 < 4 {
						tiles[x][y] = TileType::Floor;
					}
					if amount_of_water.0 + amount_of_water.1 > 0 {
						tiles[x][y] = TileType::TallGrass(0);
					}
				}
				if tiles[x][y] == TileType::Floor {
					if amount_of_flora.1 >= 1 {
						tiles[x][y] = TileType::ShortGrass(0);
					}
					if amount_of_flora.0 >= 4 {
						tiles[x][y] = TileType::ShortGrass(0);
					}
					if amount_of_flora.0 > 0 && amount_of_water.0 + amount_of_water.1 > 0 {
						tiles[x][y] = TileType::ShortGrass(0);
					} 
				}
			// Mushrooms	
			} else if (flora_type == TileType::SmallMushroom) || (flora_type == TileType::LargeMushroom){
				if tiles[x][y] == TileType::SmallMushroom {
					if amount_of_flora.0 + amount_of_flora.1 > 3 {
						tiles[x][y] = TileType::LargeMushroom;
					}
					if amount_of_flora.0 + amount_of_flora.1 < 3 {
						tiles[x][y] = TileType::Floor;
					}
				}
			if tiles[x][y] == TileType::Floor {
					if amount_of_flora.1 >= 1 {
						tiles[x][y] = TileType::SmallMushroom;
					} 
				}
			}
		}
	}

	let shore_regions = get_all_regions(TileType::ShallowWater, tiles);

	let mut best_distance = 0;
	let mut possible_connection_found: bool;

	for x in 1..width {
		for y in 1..height {
			possible_connection_found = false;
			let current_type = tiles[x][y];
			if current_type == TileType::ShortGrass(0) || current_type == TileType::TallGrass(0){
				for region in shore_regions.clone() {
					for shore_tile in region.tiles {
						let distance = ((shore_tile.0 as i32 - x as i32).pow(2) + (shore_tile.1 as i32 - y as i32).pow(2)) as f64;
						let distance = distance.sqrt() as i32 + 1;

						if distance < best_distance || !possible_connection_found {
							best_distance = distance;
							possible_connection_found = true;

							if distance < 10 {
								if current_type == TileType::ShortGrass(0) {
									tiles[x][y] = TileType::ShortGrass(distance);
								} else if current_type == TileType::TallGrass(0) {
									tiles[x][y] = TileType::TallGrass(distance);
								}
							}
						}	
					}
				}
			}
		}
	}
}

// Returns a tuple of the amount of flora surrounding a given tile. The first item is small flora, the second item is large flora.
pub fn get_amount_of_flora (x_coord: usize, y_coord: usize, flora_type: TileType, tiles: &Vec<Vec<TileType>>) -> (i32,i32) {
	let width = tiles.len();
	let height = tiles[0].len();
	let mut small_flora_count = 0;
	let mut large_flora_count = 0;
	for x in (x_coord - 1)..(x_coord + 2){
		for y in (y_coord -1)..(y_coord +2) {
			if (x < width - 1) && (y < height-1){
				if !((x == x_coord) && (y == y_coord)) {
						//grass rules
					if (flora_type == TileType::ShortGrass(0)) || (flora_type == TileType::TallGrass(0)) {
						if tiles[x][y] == TileType::ShortGrass(0) {
							small_flora_count +=1;
						}
						if tiles[x][y] == TileType::TallGrass(0) {
							large_flora_count +=1
						}
						// fungi rules
					} else if (flora_type == TileType::SmallMushroom) || (flora_type == TileType::LargeMushroom) {
						if tiles[x][y] == TileType::SmallMushroom {
							small_flora_count +=1;
						}
						if tiles[x][y] == TileType::LargeMushroom {
							large_flora_count +=1
						}
					// liquid rules	
					} else if (flora_type == TileType::ShallowWater) || (flora_type == TileType::DeepWater) {
						if tiles[x][y] == TileType::ShallowWater {
							small_flora_count +=1;
						} else if tiles [x][y] == TileType::DeepWater {
							large_flora_count +=1;
						}
					} else if (flora_type == TileType::ShallowLava) || (flora_type == TileType::DeepLava) {
						if tiles[x][y] == TileType::ShallowLava {
							small_flora_count +=1;
						} else if tiles [x][y] == TileType::DeepLava {
							large_flora_count +=1;
						}
					} 
				}
			}
		}
	}
	return (small_flora_count, large_flora_count);
}

/// Forms lakes using cellular automata
pub fn water (liquid_type: TileType, tiles: &mut Vec<Vec<TileType>>, number_generator: &mut StdRng) {
	let width = tiles.len();
	let height = tiles[0].len();

	println!("Watering...");

	let mut lake_map : Vec<Vec<TileType>> = tiles.clone();

	//fill place initial liquid tiles
	for x in 1..width-1 {
		for y in 1..height-1 {
			if is_safe(&lake_map[x][y]) {
				if number_generator.gen_range (0, 100) < 50 {
					match liquid_type {
						TileType::ShallowLava | TileType::DeepLava => lake_map[x][y] = TileType::ShallowLava,
						TileType::ShallowWater | TileType::DeepWater | _ => lake_map[x][y] = TileType::ShallowWater,
					}
				} 
			}
		}
	}

	// cellular automata
	for _i in 0..5 {
		for x in 1..width {
			for y in 1..height {
				match liquid_type {
					TileType::ShallowLava | TileType::DeepLava => {
						let amount_of_liquid = get_amount_of_flora(x, y, TileType::ShallowLava, &mut lake_map);
						if lake_map[x][y] == TileType::ShallowLava {
							if amount_of_liquid.0 >= 4 {
								lake_map[x][y] = TileType::ShallowLava;
							}
							if amount_of_liquid.0 < 2 {
								lake_map[x][y] = TileType::Floor;
							}
						} else {
							if amount_of_liquid.0 >= 5 {
								lake_map[x][y] = TileType::ShallowLava;
							}
						}
					},
					TileType::ShallowWater | TileType::DeepWater | _ => {
						let amount_of_liquid = get_amount_of_flora(x, y, TileType::ShallowWater, &mut lake_map);
						if lake_map[x][y] == TileType::ShallowWater {
							if amount_of_liquid.0 >= 4 {
								lake_map[x][y] = TileType::ShallowWater;
							}
							if amount_of_liquid.0 < 2 {
								lake_map[x][y] = TileType::Floor;
							}
						} else {
							if amount_of_liquid.0 >= 5 {
								lake_map[x][y] = TileType::ShallowWater;
							}
						}
					},
				}
			}
		}
	}

	//get lakes and reject them if they dont make requirements
	let mut lakes = get_all_regions(liquid_type, &mut lake_map);

	println!("{} lakes generated...", lakes.len());

	let mut bad_lake_counter = 0;
	lakes.retain(|a|
		if (a.size < 20) || (a.size > 600) {
			bad_lake_counter += 1;
			false
		} else {
			true
		}
	);

	println!("{} lakes rejected...", bad_lake_counter);
	
	lakes.sort_by(|a, b| a.size.cmp(&b.size));

	//put lakes on actual map
	for lake in lakes {
		for tile in lake.tiles {
			tiles[tile.0][tile.1] = lake_map[tile.0][tile.1];
		}
	}

	//put deep varieties of liquids
	match liquid_type {
		TileType::ShallowLava | TileType::DeepLava => {
			for x in 1..width {
				for y in 1..height {
					let amount_of_liquid = get_amount_of_flora(x, y, TileType::ShallowLava, tiles);
					if amount_of_liquid.0 + amount_of_liquid.1 == 8 {
						tiles[x][y] = TileType::DeepLava;
					}
				}
			}
		},
		TileType::ShallowWater | TileType::DeepWater | _ => {
			for x in 1..width {
				for y in 1..height {
					let amount_of_liquid = get_amount_of_flora(x, y, TileType::ShallowWater, tiles);
					if amount_of_liquid.0 + amount_of_liquid.1 == 8  {
						tiles[x][y] = TileType::DeepWater;
					}
				}
			}
		},	
	}
}

/// Removes wall tiles that cannot be seen by the player.
pub fn remove_unseen_walls (tiles : &mut Vec<Vec<TileType>>) {
	println!("Removing unseen walls");

	let width = tiles.len();
	let height = tiles[0].len();
	let mut flagged_points : Vec<(usize, usize)> = Vec::new();

	for x in 1..width-1 {
		for y in 1..height-1 {
			if tiles[x][y] == TileType::Wall {
				if get_surrounding_wall_count(tiles, x, y) == 8 {
					flagged_points.push((x,y));
				}
			}
		}
	}

	for point in flagged_points {
		tiles[point.0][point.1] = TileType::Empty;
	}
}
 