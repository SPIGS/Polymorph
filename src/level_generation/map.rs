use rand::{StdRng, SeedableRng};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TileType {
	Empty,
	Wall,
	Floor,
	ShortGrass(i32),
	TallGrass(i32),
	SmallMushroom,
	LargeMushroom,
	ShallowWater,
	DeepWater,
	ShallowLava,
	DeepLava,
}

#[derive(Default)]
pub struct VisionMap (pub Vec<Vec<bool>>);

#[derive(Default)]
pub struct LightMap (pub Vec<Vec<bool>>);

#[derive(Default)]
pub struct TransparencyMap (pub Vec<Vec<bool>>);

/// Returns true if the given `TileType` can be safely traversed by the player.
pub fn is_safe (tile : &TileType) -> bool {
	match tile {
		TileType::DeepLava | TileType::ShallowLava => return false,
		TileType::Wall => return false,
		_ => return true,
	}
}

/// Returns true if the given `TileType` is a liquid
pub fn is_liquid (tile : TileType) -> bool {
	match tile {
		TileType::ShallowWater | TileType::DeepWater => true,
		TileType::ShallowLava | TileType::DeepLava => true,
		_ => false,
	}
}

pub struct Seed {
	pub raw: String,
	hash: [u8; 32],
}

impl Seed {
	pub fn new (raw_text: String) -> Self {
		use sha2::{Digest, Sha256};
		let mut hasher = Sha256::default();
		hasher.input(&raw_text.as_bytes());
		let result = hasher.result();
		let mut bytes: [u8; 32] = [0; 32];
		
		for i in 0..result.len() {
			bytes[i] = result[i];
		}
		
		Seed {
			raw: raw_text,
			hash: bytes,
		}
	}

	/// Returns a 32-bit hash of this seed in the form of a u32.
	/// 
	/// Useful for random number generation using the `tcod::random` module.
	pub fn to_32_bit (&self) -> u32 {
		let mut bytes : [u8; 4] = [0; 4];
		for i in 0..4 {
			bytes[i] = self.hash[i];
		}

		return u32::from_be_bytes(bytes);
	}

	/// Returns a full 256-bit hash of this seed in the form of a [u8; 32].
	/// 
	/// Useful for random number generation using `SeedableRng` from the `rand` module.
	pub fn to_256_bit (&self) -> [u8; 32]{
		return self.hash;
	}
}

#[derive(Clone)]
pub struct Region {
	pub id: String,
	pub tiles : Vec<(usize, usize)>,
	pub edge_tiles : Vec<(usize, usize)>,
	pub connected_regions : Vec<String>,
	pub size: usize,
	pub is_main_region: bool,
	pub is_connected_to_main_region: bool,
}

impl Region {
	pub fn is_tile_in_region (&mut self, other_tile : &(usize, usize)) -> bool {
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

pub struct Map {
	pub tiles: Vec<Vec<TileType>>,
	pub width: usize,
	pub height: usize,
	pub seed: Seed,
	pub number_generator: StdRng,
	pub spawn_point: (i32,i32),
	pub exit_point: (i32,i32),
	pub transparency : Vec<Vec<bool>>,
}

impl Map {
	pub fn new (width: usize, height: usize, raw_seed: String) -> Self {
		
		let level_seed = Seed::new(raw_seed);
		let number_generator : StdRng = SeedableRng::from_seed(level_seed.to_256_bit());

		Map {
			tiles: vec![vec![TileType::Wall; height]; width],
			width: width,
			height: height,
			seed: level_seed,
			number_generator: number_generator,
			spawn_point: (0,0),
			exit_point: (0,0),
			transparency : Vec::default(),
		}
	}

	pub fn generate (&mut self) {
		use crate::level_generation::cellular_automata;
		
		cellular_automata::random_fill(&mut self.tiles, &mut self.number_generator);
		
		for _i in 0..5 {
			cellular_automata::perform_generation(&mut self.tiles);
		}

		cellular_automata::clean_up_walls(&mut self.tiles, 11);
		cellular_automata::clean_up_floors(&mut self.tiles);
		cellular_automata::smooth(&mut self.tiles);

		if cellular_automata::get_all_regions(TileType::Floor, &mut self.tiles).len() > 1 {
			if cellular_automata::connect_floor_regions(&mut self.tiles) == false {
				println!("Rejected level; generating again");
				self.generate();
			} else {
				cellular_automata::smooth(&mut self.tiles);
				cellular_automata::clean_up_walls(&mut self.tiles, 5);
				cellular_automata::water(TileType::ShallowWater, &mut self.tiles, &mut self.number_generator);
				cellular_automata::plant(&mut self.tiles, &mut self.number_generator, TileType::ShortGrass(0), 25);
				cellular_automata::grow(&mut self.tiles, TileType::ShortGrass(0));
				cellular_automata::remove_unseen_walls(&mut self.tiles);
			}
		} else {
			cellular_automata::smooth(&mut self.tiles);
			cellular_automata::clean_up_walls(&mut self.tiles, 5);
			cellular_automata::water(TileType::ShallowLava, &mut self.tiles, &mut self.number_generator);
			cellular_automata::plant(&mut self.tiles, &mut self.number_generator, TileType::ShortGrass(0), 25);
			cellular_automata::grow(&mut self.tiles, TileType::ShortGrass(0));
			cellular_automata::remove_unseen_walls(&mut self.tiles);
		}

		self.set_spawn_and_exit();

		let mut new_transparency = vec![vec![false; self.height]; self.width];

		for x in 0..self.width {
			for y in 0..self.height {
				match self.tiles[x][y] {
					TileType::Wall => new_transparency[x][y] = false,
					TileType::TallGrass(_i) => new_transparency[x][y] = false,
					_ => new_transparency[x][y] = true,
				}
			}
		}

		self.transparency = new_transparency;
	}

	fn set_spawn_and_exit (&mut self) {
		use rand::seq::sample_slice;

		let mut safe_tiles = self.get_largest_safe_region(&self.tiles.clone());

		safe_tiles.retain(|a| match a.2 {
			TileType::Floor => true,
			TileType::TallGrass(_distance) => true,
			TileType::ShortGrass(_distance) => true,
			_ => false,	
		});


		// out of all the safe tiles, pick a random tile to be the players spawn point on the map.
		let player_spawn_tile : (usize, usize, TileType) = sample_slice(&mut self.number_generator, &safe_tiles, 1)[0];

		// remove the tile that will become the player's spawn point from the list of safe tiles.
		safe_tiles.retain(|&a| a != player_spawn_tile);

		//remove the tiles that are not a minimum distance away from the spawn point
		safe_tiles.retain(|&a| 
			(((a.0 as i32 - player_spawn_tile.0 as i32).pow(2) + (a.1 as i32 - player_spawn_tile.1 as i32).pow(2)) as f64) > 1000.0
		);
	
		let exit_tile : (usize, usize, TileType) = sample_slice(&mut self.number_generator, &safe_tiles, 1)[0];

		self.spawn_point = (player_spawn_tile.0 as i32, player_spawn_tile.1 as i32);
		self.exit_point = (exit_tile.0 as i32, exit_tile.1 as i32);
	}

	fn get_largest_safe_region (&mut self, tile_map : &Vec<Vec<TileType>>) -> Vec<(usize, usize, TileType)> {
		let mut regions: Vec<Region> = Vec::new();
		let mut mapflags: Vec<Vec<i32>> = vec![vec![0; self.height]; self.width];

		for x in 1..(self.width) {
			for y in 1..(self.height) {

				if (mapflags[x as usize][y as usize] == 0) && is_safe(&tile_map[x][y]){
					let new_region = self.get_safe_region((x,y), tile_map);

					for tile in new_region.tiles.iter(){
						
						mapflags[tile.0 as usize][tile.1 as usize] = 1;
					}
					regions.push(new_region);
				}
			}
		}

		regions.sort_by(|a,b| b.tiles.len().cmp(&a.tiles.len()));

		let region = regions[0].clone();
		let mut safe_tiles: Vec<(usize, usize, TileType)> = Vec::new();

		for tile in region.tiles {
			let tile_type = self.tiles[tile.0 as usize][tile.1 as usize];

			safe_tiles.push((tile.0, tile.1, tile_type));
		}

		return safe_tiles;
	}

	fn get_safe_region (&mut self, start_tile: (usize, usize), tiles : &Vec<Vec<TileType>>) -> Region  {
		let mut region_tiles: Vec<(usize, usize)> = Vec::new();
		let mut mapflags:Vec<Vec<u8>> = vec![vec![0; self.height as usize]; self.width as usize];

		let mut queue: VecDeque<(usize,usize)> = VecDeque::new();

		queue.push_back(start_tile);
		mapflags[start_tile.0][start_tile.0] = 1;
		while !queue.is_empty(){

			let tile = queue.pop_front().unwrap();
			
			region_tiles.push(tile);
			for x in (tile.0-1)..(tile.0 +2){
				for y in (tile.1 - 1).. (tile.1 + 2){
					if ((x < self.width) && (y < self.height)) && (y == tile.1 || x == tile.0){
						if (mapflags[x][y] == 0) && is_safe(&tiles[x][y]) {
							mapflags[x][y] = 1;
							queue.push_back((x, y));
						}
					}
				}
			}
		}
		use crate::level_generation::cellular_automata::is_edge_tile;
		use crate::application::get_current_time_millis;

		let size = region_tiles.len();
		let mut edge_tiles = region_tiles.clone();
		edge_tiles.retain(|a| is_edge_tile(a.0 as i32, a.1 as i32, &mut self.tiles));

		let region = Region {
			id: format!("{}", get_current_time_millis()),
			tiles: region_tiles,
			edge_tiles: edge_tiles,
			connected_regions : Vec::new(),
			size: size,
			is_main_region: false,
			is_connected_to_main_region: false,
		};
		return region
	}

}