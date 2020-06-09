use rand::{Rng, StdRng};
use super::map::tile::TileType;
use super::features::{make_spider_nest, make_camp, FeatureType};

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
	pub fn is_tile_in_region (&self, other_tile : &(i32, i32)) -> bool {
		return self.tiles.contains(other_tile);
	}
}

pub struct CellularGenerator {
	initial_fill_percent : i32,
	clean_up_threshold : usize,
	generations : usize,
	smoothing : usize,
	floral_density : i32,
	flora : TileType,
	wall : TileType,
	floor : TileType,
	liquid : TileType,
	min_lake_size : usize,
	max_lake_size : usize,
	features : FeatureType,

}

impl CellularGenerator {
	pub fn new (initial_fill : i32, clean_up_threshold : usize, number_of_generations : usize, smoothing : usize) -> Self{
		CellularGenerator {
			initial_fill_percent : initial_fill,
			clean_up_threshold : clean_up_threshold,
			generations : number_of_generations,
			smoothing : smoothing,
			floral_density : 0,
			flora : TileType::Empty,
			wall : TileType::Empty,
			floor : TileType::Empty,
			liquid : TileType::Empty,
			min_lake_size : 0,
			max_lake_size : 0,
			features : FeatureType::NoFeatures,
		}
	}

	pub fn set_flora (&mut self, flora_type : TileType, density : i32) {
		self.flora = flora_type;
		self.floral_density = density;
	}

	pub fn set_walls_floors (&mut self, wall_type : TileType, floor_type : TileType) {
		self.wall = wall_type;
		self.floor = floor_type;
	}

	pub fn set_liquid (&mut self, liquid_type : TileType, min_size : usize, max_size : usize) {
		self.liquid = liquid_type;
		self.min_lake_size = min_size;
		self.max_lake_size = max_size;
	}

	pub fn set_features (&mut self, features : FeatureType) {
		self.features = features;
	}

	pub fn generate (&self, width : usize, height : usize, tiles : &mut Vec<TileType>, rng: &mut StdRng) {
		random_fill(tiles, rng, self.wall, self.floor, self.initial_fill_percent);
		add_map_edges(width, height, tiles, self.wall);
		for _i in 0..self.generations {
			perform_generation(width, height, tiles, self.wall, self.floor);
		}

		clean_up_regions(width, height, tiles, self.wall, self.floor, self.clean_up_threshold);
		clean_up_regions(width, height, tiles, self.floor, self.wall, self.clean_up_threshold);

		for _i in 0..self.smoothing {
			smooth(width, height, tiles, self.wall, self.floor);
		}
		
		clean_up_regions(width, height, tiles, self.floor, self.wall, self.clean_up_threshold);
		
		let mut possible_camps : Vec<Region> = Vec::new();
		let mut possible_nests : Vec<Region> = Vec::new();
		if self.features != FeatureType::NoFeatures {
			let regions = get_all_regions(width, height, self.floor, tiles);
			for region in regions {
				if region.size > 100 {
					possible_camps.push(region.clone());
				} else if region.size < 20 {
					possible_nests.push(region.clone());
				}
			}
		} 
		
		if connect_floor_regions(width, height, tiles, self.floor) {
			error!("Rejected level; generating again");
			self.generate(width, height, tiles, rng);
		} else {
			clean_up_regions(width, height, tiles, self.wall, self.floor, self.clean_up_threshold);		
			add_map_edges(width, height, tiles, self.wall);
			
			remove_unseen_walls(width, height, tiles, self.wall, self.floor);
			
			if self.liquid != TileType::Empty {
				make_lakes(width, height, self.liquid, self.floor, tiles, &possible_nests, self.min_lake_size, self.max_lake_size, rng);
			}
			if self.flora != TileType::Empty {
				plant(width, height, tiles, rng, self.flora, self.floor, self.floral_density);
				grow(width, height, tiles, self.flora, self.floor);
			}

			for region in possible_camps.clone() {
				make_camp(width, tiles, region);
			}

			for region in possible_nests.clone() {
				make_spider_nest(width, tiles, region, rng);
			}
			
			if self.liquid != TileType::Empty {
				add_lake_depth(width, height, self.liquid, tiles);
			}
		}
	}
}

fn random_fill (tiles : &mut Vec<TileType>, rng: &mut StdRng, wall : TileType, floor : TileType, initial_fill_percent : i32) {
    info!("Seeding level...");

    for i in 0..tiles.len() {
        if rng.gen_range(0, 100) < initial_fill_percent {
			tiles[i] = wall;
		} else {
			tiles[i] = floor;
		}
    }

}

/// Performs a single generation of the cellular automata 
/// according to the B5678/S45678 rule.
pub fn perform_generation (width : usize, height : usize, tiles : &mut Vec<TileType>, wall : TileType, floor : TileType) {
	info!("Generating...");

	for x in 1..width-1 {
		for y in 1..height-1 {
			let surrounding_wall_count = get_surrounding_neighbor_count(width, height, tiles, wall, x as i32, y as i32);

			match tiles[x+y*width] {
				_ if tiles[x+y*width] == wall => {
					if surrounding_wall_count >= 4 {
						tiles[x+y*width] = wall;
					}
					if surrounding_wall_count < 2 {
						tiles[x+y*width] = floor;
					}
				},
				_ => {
					if surrounding_wall_count >= 5 {
						tiles[x+y*width] = wall;
					}
				}
			}
		}
	}
}

///Removes wall or floor regions that don't meet size requirements.
pub fn clean_up_regions (width : usize, height : usize, tiles : &mut Vec<TileType>, t : TileType, r : TileType, size_threshold: usize) {
	info!("Cleaning...");
	let mut wall_regions = get_all_regions(width, height, t, tiles);

	let mut excluded_regions : Vec<Region> = Vec::new();

	//exclude bad wall regions
	for i in (0..wall_regions.len()).rev() {
		if wall_regions[i].tiles.len() <= size_threshold {
			excluded_regions.push(wall_regions.remove(i));
		}
	}

	debug!("{} excluded regions;", excluded_regions.len());

	for region in excluded_regions {
		for tile in region.tiles {
			let x = tile.0 as usize;
			let y = tile.1 as usize;

			tiles[x+y*width] = r;
		}
	}
}

/// Applies a different cellular ruleset to smooth out jagged edges.
pub fn smooth (width : usize, height : usize, tiles : &mut Vec<TileType>, wall : TileType, floor : TileType) {
	info!("Smoothing...");

	for x in 1..width-1 {
		for y in 1..height-1 {
			let surrounding_wall_count = get_surrounding_neighbor_count(width, height, tiles, wall, x as i32, y as i32);

			if surrounding_wall_count > 4 {
				tiles[x+y*width] = wall;
			} else if surrounding_wall_count < 4 {
				tiles[x+y*width] = floor;
			}
		}
	}
}

pub fn add_map_edges (width : usize, height : usize, tiles : &mut Vec<TileType>, wall : TileType) {
	for x in 0..width {
		if tiles [x + 0 * width] != wall {
			tiles [x + 0 * width] = wall;
		}
		if tiles [x + (height - 1) * width] != wall {
			tiles[x + (height - 1) * width] = wall;
		}
	}

	for y in 0..height {
		if tiles [0 + y * width] != wall {
			tiles [0 + y * width] = wall;
		}
		if tiles [(width - 1) +y * width] != wall {
			tiles[(width -1) + y * width] = wall;
		}
	}
}

/// Forms lakes using cellular automata. Panics if the given type is not a liguid.
pub fn make_lakes (width : usize, height : usize, liquid_type: TileType, floor : TileType, tiles: &mut Vec<TileType>, marked_regions : &Vec<Region>, min_size : usize, max_size : usize, number_generator: &mut StdRng) {
	use super::map::tile::{shallow_liquid_variant};
	info!("Forming Lakes...");

	let shallow_variant = shallow_liquid_variant(liquid_type);

	let mut lake_map : Vec<TileType> = tiles.clone();

	//fill place initial liquid tiles
	for x in 1..width-1 {
		for y in 1..height-1 {
			if lake_map[x+y*width] == floor {
				if number_generator.gen_range (0, 100) < 50 {
					lake_map[x+y*width] = shallow_variant;
				} 
			}
		}
	}

	// cellular automata
	for _i in 0..5 {
		for x in 1..width {
			for y in 1..height {
				let idx = x+y*width;
				let amount_of_liquid = get_surrounding_neighbor_count(width, height, &mut lake_map, shallow_variant, x as i32, y as i32);
				if lake_map[idx] == shallow_variant {
					if amount_of_liquid >= 4 {
						lake_map[idx] = shallow_variant;
					}
					if amount_of_liquid < 2 {
						lake_map[idx] = floor;
					}
				} else {
					if lake_map[idx] == floor && amount_of_liquid >= 5 {
						lake_map[idx] = shallow_variant;
					}
				}
			}
		}
	}

	//reject lakes if they dont make requirements
	let mut lakes = get_all_regions(width, height, liquid_type, &mut lake_map);

	debug!("{} lakes generated...", lakes.len());
	let mut bad_lake_counter = 0;
	lakes.retain(|a|
		if (a.size < min_size) || (a.size > max_size) {
			bad_lake_counter += 1;
			false
		} else {
			true
		}
	);

	let mut foo = 0;
	// reject lakes if they would intrude on feature regions
	lakes.retain(|a|
		{
			let mut retain = true;
			for edge_tile in &a.edge_tiles {
				for region in marked_regions {
					if region.is_tile_in_region(&edge_tile) {
						retain = false;
						bad_lake_counter += 1;
						foo += 1;
						break;
					} else {
						continue;
					}
				}
				if !retain {
					break;
				}
			}
			retain
		});

	debug!("{} lakes rejected...", bad_lake_counter);	

	//put lakes on actual map
	for lake in lakes {
		for tile in lake.tiles {
			let idx = (tile.0+tile.1*width as i32) as usize;
			tiles[idx] = lake_map[idx];
		}
	}
}

pub fn add_lake_depth (width : usize, height : usize, liquid_type: TileType, tiles: &mut Vec<TileType>) {
	use super::map::tile::{shallow_liquid_variant, deep_liquid_variant};
	let shallow_variant = shallow_liquid_variant(liquid_type);
	let deep_variant = deep_liquid_variant(liquid_type);

	//put deep varieties of liquids
	for x in 1..width {
		for y in 1..height {
			let amount_of_shallow_liquid = get_surrounding_neighbor_count(width, height, tiles, shallow_variant, x as i32, y as i32);
			let amount_of_deep_liquid = get_surrounding_neighbor_count(width, height, tiles, deep_variant, x as i32, y as i32);
			if amount_of_shallow_liquid + amount_of_deep_liquid == 8 {
				tiles[x+y*width] = deep_variant;
			}
		}
	}
}

/// Like `random_fill` but places a type of foliage on `TileType::Floor` according to a given density.
/// Panics if the given type is not foliage.
pub fn plant (width : usize, height : usize, tiles: &mut Vec<TileType>, number_generator: &mut StdRng, flora_type: TileType, floor : TileType, density: i32) {
	use super::map::tile::small_foliage_variant;
	info!("Planting...");

	let small_variant = small_foliage_variant(flora_type);

	for x in 1..width-1 {
		for y in 1..height-1 {
			let idx = x+y*width;
			if tiles[idx] == floor {
				let chance = number_generator.gen_range(0, 100);
				if chance < density {
					tiles[idx] = small_variant;
				}
			}
		}
	}
}

/// Performs a cellular generation on a type of foilage. And calculates how far each grass tile is from water
/// Panics if the given flora_type is not actually foliage.
pub fn grow (width : usize, height : usize, tiles: &mut Vec<TileType>, flora_type: TileType, floor : TileType) {
	use super::map::tile::{small_foliage_variant, large_foliage_variant};
	info!("Growing...");

	let large_variant = large_foliage_variant(flora_type);
	let small_variant = small_foliage_variant(flora_type);

	for x in 1..width-1 {
		for y in 1..height-1 {
			let amt_small_flora = get_surrounding_neighbor_count(width, height, tiles, small_variant, x as i32, y as i32);
			let amt_large_flora = get_surrounding_neighbor_count(width, height, tiles, large_variant, x as i32, y as i32);
			let idx = x+y*width;
			//Grass
			if small_variant == TileType::ShortGrass(0) {
				let amt_of_water = get_surrounding_neighbor_count(width, height, tiles, TileType::ShallowWater, x as i32, y as i32);
				if tiles[idx] == TileType::ShortGrass(0) {	
					if amt_small_flora + amt_large_flora >= 3 {
						tiles[idx] = TileType::TallGrass(0);
					}
					if amt_small_flora + amt_large_flora < 4 {
						tiles[idx] = floor;
					}
					if amt_of_water > 0 {
						tiles[idx] = TileType::TallGrass(0);
					}
				}
				if tiles[idx] == floor {
					if amt_large_flora >= 1 {
						tiles[idx] = TileType::ShortGrass(0);
					}
					if amt_small_flora >= 4 {
						tiles[idx] = TileType::ShortGrass(0);
					}
					if amt_small_flora > 0 && amt_of_water > 0 {
						tiles[idx] = TileType::ShortGrass(0);
					} 
				}
			// Mushrooms	
			} else if small_variant == TileType::SmallMushroom {
				if tiles[idx] == TileType::SmallMushroom {
					if amt_small_flora + amt_large_flora > 3 {
						tiles[idx] = TileType::LargeMushroom;
					}
					if amt_small_flora + amt_large_flora < 3 {
						tiles[idx] = floor;
					}
				}
			if tiles[idx] == floor {
					if amt_large_flora >= 1 {
						tiles[idx] = TileType::SmallMushroom;
					} 
				}
			}
		}
	}

	if small_variant == TileType::ShortGrass(0) {
		let shore_regions = get_all_regions(width, height, TileType::ShallowWater, tiles);

		let mut best_distance = 0;
		let mut possible_connection_found: bool;

		for x in 1..width {
			for y in 1..height {
				let idx = x+y*width;
				possible_connection_found = false;
				let current_type = tiles[idx];
				if current_type == small_variant || current_type == large_variant {
					for region in shore_regions.clone() {
						for shore_tile in region.tiles {
							let distance = ((shore_tile.0 as i32 - x as i32).pow(2) + (shore_tile.1 as i32 - y as i32).pow(2)) as f64;
							let distance = distance.sqrt() as i32 + 1;

							if distance < best_distance || !possible_connection_found {
								best_distance = distance;
								possible_connection_found = true;

								if distance < 10 {
									if current_type == TileType::ShortGrass(0) {
										tiles[idx] = TileType::ShortGrass(distance);
									} else if current_type == TileType::TallGrass(0) {
										tiles[idx] = TileType::TallGrass(distance);
									}
								}
							}	
						}
					}
				}
			}
		}
	}
}

/// Removes wall tiles that cannot be seen by the player.
pub fn remove_unseen_walls (width : usize, height: usize, tiles : &mut Vec<TileType>, wall : TileType, floor : TileType) {
	debug!("Removing unseen walls");

	let mut flagged_points : Vec<(usize, usize)> = Vec::new();

	for x in 0..width {
		for y in 0..height {
			if tiles[x+y*width] == wall {
				if get_surrounding_neighbor_count(width, height, tiles, floor, x as i32, y as i32) == 0 {
					flagged_points.push((x,y));
				}
			}
		}
	}

	for point in flagged_points {
		tiles[point.0+point.1*width] = TileType::Empty;
	}
}

/// Attempts to connect separated floor regions. Returns true if successful, returns false
/// if the paths calculated are too long.
pub fn connect_floor_regions (width : usize, height : usize, tiles: &mut Vec<TileType>, floor : TileType) -> bool {
	use bracket_lib::prelude::{Bresenham, Point};

	let mut reject_level = false;

	let mut regions = get_all_regions(width, height, floor, tiles);
	regions.sort_by(|a,b| b.size.cmp(&a.size));

	if regions.len() > 1 {
		let current_largest = regions.remove(0);
		let mut closest_distance = 0;
		let mut point_a : (i32, i32) = (0,0);
		let mut point_b : (i32, i32) = (0,0);
		let mut looking_for_closest = false;

		//find the closest point in another region other than itself
		for region in regions {
			for edge_tile_b in region.edge_tiles {
				for edge_tile_a in &current_largest.edge_tiles {
					let distance = (edge_tile_a.0 - edge_tile_b.0).pow(2) + (edge_tile_a.1 - edge_tile_b.1).pow(2);

					if distance < closest_distance || !looking_for_closest {
						looking_for_closest = true;
						closest_distance = distance;
						point_a = *edge_tile_a;
						point_b = edge_tile_b;
					}
				}
			}
		}

		// the path is too far; just regen the level
		if closest_distance > 10000 {
			return true;
		}

		let path = Bresenham::new(Point::from_tuple(point_a), Point::from_tuple(point_b));
		
		// connect the two regions
		path.for_each(|x| dig_circle(width, height, x.to_tuple(), 2, tiles, floor));
		
		//recurse
		reject_level = connect_floor_regions(width, height, tiles, floor);
	}
	return reject_level;
}

/// Digs a circle of a given radius of floors at a point 
pub fn dig_circle (width : usize, height : usize, tile : (i32, i32), radius: i32, tiles: &mut Vec<TileType>, floor : TileType) {
	for x in -radius..radius {
		for y in -radius..radius {
			if x*x + y*y <= radius*radius {
				let dig_x = tile.0 + x;
				let dig_y = tile.1 + y;
				if (dig_x >= 0 && dig_x < width as i32) && (dig_y >= 0 && dig_y < height as i32) {
					let idx = dig_x as usize + dig_y as usize * width;
					
					tiles[idx] = floor;
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

/// Returns true of the given coordates are of a tile on the edge of a region.
pub fn is_edge_tile (width: usize, height : usize, x: i32, y: i32, tiles : &mut Vec<TileType>) -> bool {
	for dx in (x - 1)..(x + 2) {
		for dy in (y-1)..(y+2) {
			if (dx <= 0 || dx >= width as i32) || (dy <= 0 || dy >= height as i32) {
				return true;
			} else {
				let idx = dx as usize + dy as usize * width;
				if tiles[x as usize+y as usize*width] != tiles[idx] {
					return true
				} else {
					continue;
				}
			}

		}
	}
	return false;
}

/// Returns the number of neighbors that match a given type out of the 8 neighbors of a given tile.
fn get_surrounding_neighbor_count (width : usize, height : usize, tiles : &mut Vec<TileType>, neighbor_type : TileType, x: i32, y: i32) -> i32 {
	let mut count = 0;
	for dx in (x - 1)..(x + 2) {
		for dy in (y - 1)..(y + 2) {
			if !((dx == x) && (dy == y)) && !(dx < 0 || dx >= width as i32) && !(dy < 0 || dy >= height as i32) {
				let idx = (dx + dy * width as i32) as usize;
				if tiles[idx] == neighbor_type {
					count += 1;
				}
			
			}
		}
	}
	return count;
}
