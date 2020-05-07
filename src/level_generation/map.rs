use rand::{StdRng, SeedableRng};
use super::cellular;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TileType {
    Empty,
    Floor,
    Wall,
}
impl Default for TileType {
    fn default() -> Self { TileType::Empty }
}

#[derive(Debug)]
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

#[derive(Debug, PartialEq)]
pub enum MapType {
	Cavern,
}

#[derive(Debug)]
pub struct Map {
    pub width : usize,
	pub height : usize,
	pub map_type : MapType,
	pub raw_seed: String,
	hashed_seed: Seed,
	rng : StdRng,
    pub tiles : Vec<TileType>,
    pub transparency_map : Vec<f32>,
}

impl Default for Map {
	fn default() -> Self {
		let hashed_seed = Seed::new(String::from("null").clone());
		let rng : StdRng = SeedableRng::from_seed(hashed_seed.to_256_bit());

		Map {
            width : 10,
			height : 10,
			map_type : MapType::Cavern,
			raw_seed : String::from("null"),
			hashed_seed : hashed_seed,
			rng : rng,
            tiles : vec![TileType::Empty; 10*10],
            transparency_map : vec![0.0; 10*10],
        }
	}
}

impl Map {
    pub fn new (width : usize, height : usize, raw_seed : String, map_type : MapType) -> Self {
		
		let hashed_seed = Seed::new(raw_seed.clone());
		let number_generator : StdRng = SeedableRng::from_seed(hashed_seed.to_256_bit());
        Map {
            width : width,
			height : height,
			map_type : map_type,
			raw_seed : raw_seed,
			hashed_seed : hashed_seed,
			rng : number_generator,
            tiles : vec![TileType::Empty; width*height],
            transparency_map : vec![0.0; width*height],
        }
	}
	
	pub fn generate (&mut self) {
		info!("Generating map...");
		cellular::generate(self.width, self.height, &mut self.tiles, &mut self.transparency_map, &mut self.rng);
	}
}