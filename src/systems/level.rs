use specs::{ReadStorage, WriteStorage, System, Entities, Write};

use crate::components::basic::{Position, Character, Description, ColorLerp, Light, LightMask};
use crate::components::tags::{PlayerTag, TileTag, DirtyFlag, StaticFlag};

use crate::state::{CurrentWorldAction};

use crate::level_generation::map::*;
use rand::{Rng, StdRng};
use tcod::{colors, Color};

use crate::systems::render::ColorWithAlpha;

use crate::systems::light::{StaticLightMap, VisionMap, TransparencyMap, DynamicLightMap};

#[derive(Default)]
pub struct ExitPosition (pub (i32,i32));

#[derive(Default)]
pub struct SpawnPosition (pub (i32, i32));

pub struct ClearLevelSystem;

impl<'a> System<'a> for ClearLevelSystem {
	type SystemData = (
			ReadStorage<'a, PlayerTag>,
			Write<'a, CurrentWorldAction>,
			Entities<'a>,
	);

	fn run (&mut self, (player_tags, mut _current_world_action, entities): Self::SystemData) {
		use specs::Join;
		for (entity, _player_tag) in (&entities, !&player_tags).join() {
			let _ = entities.delete(entity);
		}
	}
}

pub struct NewLevelSystem {
	pub seed: String,
	pub current_level_number: i32,
	pub current_level: Map,
}

impl NewLevelSystem {
	pub fn new (seed: String) -> Self {
		NewLevelSystem {
			seed: seed,
			current_level_number: 0,
			current_level: Map::new(1, 1, String::from("")),
		}
	}
}

impl<'a> System<'a> for NewLevelSystem {
	type SystemData = (
		WriteStorage<'a, Position>,
		WriteStorage<'a, Character>,
		WriteStorage<'a, Description>,
		WriteStorage<'a, ColorLerp>,
		ReadStorage<'a, PlayerTag>,
		WriteStorage<'a, TileTag>,
		WriteStorage<'a, DirtyFlag>,
		WriteStorage<'a, StaticFlag>,
		WriteStorage<'a, Light>,
		WriteStorage<'a, LightMask>,
		Write<'a, StaticLightMap>,
		Write<'a, DynamicLightMap>,
		Write<'a, TransparencyMap>,
		Write<'a, VisionMap>,
		Write<'a, SpawnPosition>,
		Write<'a, ExitPosition>,
		Write<'a, CurrentWorldAction>,
		Entities<'a>,
	);

	fn run (&mut self, (mut positions, mut characters, mut descriptions, mut color_lerps, player_tags, mut tile_tags, mut dirty_flags, mut static_flags, mut lights, mut light_masks, mut static_light_map, mut dynamic_light_map, mut transparency_map, mut vision_map, mut spawn_position, mut exit_position, mut current_world_action, entities): Self::SystemData) {
		let map_width = 100;
		let map_height = 100;
		self.current_level_number += 1;
		self.current_level = Map::new(map_width, map_height, format!("{}{}", self.seed, self.current_level_number));
		self.current_level.generate();
		transparency_map.0 = self.current_level.transparency.clone();
		static_light_map.0 = vec![vec![ColorWithAlpha::new_from_tcod(colors::BLACK, 0.0); map_height]; map_width];
		dynamic_light_map.0 = vec![vec![ColorWithAlpha::new_from_tcod(colors::BLACK, 0.0); map_height]; map_width];
		vision_map.0 = vec![vec![false; map_height]; map_width];


		spawn_position.0 = self.current_level.spawn_point;

		use specs::Join;
		for (position, _player_tag) in (&mut positions, &player_tags).join() {
			position.x = self.current_level.spawn_point.0;
			position.y = self.current_level.spawn_point.1;
		}

		exit_position.0 = self.current_level.exit_point;

		let _ = entities.build_entity()
			.with(Position::new(self.current_level.exit_point.0, self.current_level.exit_point.1), &mut positions)
			.with(Character::new('>', colors::RED, colors::LIGHT_GREY), &mut characters)
			.with(Description::new(String::from("Downwards Staircase"), String::from("A staircase that descends to the level below.")), &mut descriptions)
			.build();

		let map_width = self.current_level.width;
		let map_height = self.current_level.height;

		for x in 0..map_width {
			for y in 0..map_height {
				match self.current_level.tiles[x as usize][y as usize] {
					TileType::Floor => make_floor(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut descriptions, &mut light_masks, &mut tile_tags, &mut self.current_level.number_generator, x as i32, y as i32),
					TileType::Wall => make_wall(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut descriptions, &mut light_masks, &mut tile_tags, x as i32, y as i32),
					TileType::ShortGrass(distance) => {make_short_grass(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut descriptions, &mut light_masks, &mut tile_tags, &mut self.current_level.number_generator, x, y, distance)},
					TileType::TallGrass(distance) => {make_tall_grass(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut descriptions, &mut light_masks, &mut tile_tags, &mut self.current_level.number_generator, x, y, distance)},
					TileType::SmallMushroom => {make_small_mushroom(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut lights, &mut descriptions, &mut tile_tags, &mut self.current_level.number_generator, x as i32, y as i32)},
					TileType::LargeMushroom => {make_large_mushroom(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut lights, &mut descriptions, &mut tile_tags, &mut self.current_level.number_generator, x as i32, y as i32)},
					TileType::ShallowWater => {make_shallow_water(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut descriptions, &mut light_masks, &mut color_lerps, &mut tile_tags, &mut self.current_level.number_generator, x as i32, y as i32)},
					TileType::ShallowLava => {make_shallow_lava(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut lights, &mut descriptions, &mut color_lerps, &mut tile_tags, &mut self.current_level.number_generator, x as i32, y as i32)},
					TileType::DeepWater => {make_deep_water(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut descriptions, &mut light_masks, &mut color_lerps, &mut tile_tags, &mut self.current_level.number_generator, x as i32, y as i32)},
					TileType::DeepLava => {make_deep_lava(&entities, &mut positions, &mut characters, &mut dirty_flags, &mut static_flags, &mut lights, &mut descriptions, &mut color_lerps, &mut tile_tags, &mut self.current_level.number_generator, x as i32, y as i32)},
					TileType::Empty => {},
				}
			}
		}
	}
}

fn make_floor (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>, 
				descriptions: &mut WriteStorage<Description>, light_masks : &mut WriteStorage<LightMask>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: i32, y: i32) {
	let soil_chance = number_generator.gen_range(0, 100);
	let floor = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(LightMask, light_masks)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();

	if soil_chance < 90 {
		let _ = characters.insert(floor, Character::new(' ', colors::WHITE, colors::BLACK));
		let _ = descriptions.insert(floor, Description::new(String::from("Dirt"), String::from("A patch of smooth, soft dirt.")));
	} else {
		let soil_chance = number_generator.gen_range(0, 100);
		let _ = descriptions.insert(floor, Description::new(String::from("Dirt"), String::from("A patch of coarse gravel.")));
		
		if soil_chance < 50 {
			let _ = characters.insert(floor, Character::new(',', colors::LIGHT_GREY, colors::BLACK));
		} else {
			let _ = characters.insert(floor, Character::new('.', colors::LIGHT_GREY, colors::BLACK));
		}
	}
}

fn make_wall (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>,
				descriptions: &mut WriteStorage<Description>, light_masks : &mut WriteStorage<LightMask>, tile_tags: &mut WriteStorage<TileTag>, x: i32, y: i32) {
	let _ = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(Character::new('#', colors::WHITE, colors::BLACK), characters)
		.with(Description::new(String::from("Cavern Wall"), String::from("A large, flat wall of solid rock.")), descriptions)
		.with(LightMask, light_masks)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();
}

fn make_short_grass (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>,
				descriptions: &mut WriteStorage<Description>, light_masks : &mut WriteStorage<LightMask>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: usize, y: usize, distance: i32) {
	let mut color = colors::GREEN;

	// set the healthy color
	let color_healthy : Color;
	if number_generator.gen_range(0,100) < 50 {
		color_healthy = colors::GREEN;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color_healthy = Color{b:0, g:168, r:100};
		} else {
			color_healthy = Color{b:80, g:180, r:0};
		}
	}

	//set the unhealthy color
	let color_unhealthy : Color;
	if number_generator.gen_range(0,100) < 50 {
		color_unhealthy = colors::BRASS;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color_unhealthy = Color{b:0, g:106, r:91};
		} else {
			color_unhealthy = Color{b:0, g:137, r:155};
		}
	}

	let mut lerp = ColorLerp::new(color_healthy, color_unhealthy, 100.0, 0.0);

	if distance == 0 {
		color = color_unhealthy
	} else {
		for _i in 0..distance {
			lerp.lerp_hue(1.0);
			color = lerp.current_color;
		}
	}

	let _ = entities.build_entity()
		.with(Position::new(x as i32, y as i32), positions)
		.with(Character::new('\u{FD}', color, colors::BLACK), characters)
		.with(Description::new(String::from("Grass"), String::from("A patch of short cave grass.")), descriptions)
		.with(LightMask, light_masks)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();
}

fn make_tall_grass (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>, 
				descriptions: &mut WriteStorage<Description>, light_masks : &mut WriteStorage<LightMask>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: usize, y: usize, distance: i32) {
	let character: char;
	let mut color = colors::GREEN;

	// set the character
	if number_generator.gen_range(0,100) < 70 {
		character = '\u{F4}';
	} else {
		character = '\u{F5}';
	}

	// set the healthy color
	let color_healthy : Color;
	if number_generator.gen_range(0,100) < 50 {
		color_healthy = colors::GREEN;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color_healthy = Color{b:0, g:168, r:100};
		} else {
			color_healthy = Color{b:80, g:180, r:0};
		}
	}

	//set the unhealthy color
	let color_unhealthy : Color;
	if number_generator.gen_range(0,100) < 50 {
		color_unhealthy = colors::BRASS;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color_unhealthy = Color{b:0, g:106, r:91};
		} else {
			color_unhealthy = Color{b:0, g:137, r:155};
		}
	}

	let mut lerp = ColorLerp::new(color_healthy, color_unhealthy, 100.0, 0.0);

	if distance == 0 {
		color = color_unhealthy
	} else {
		for _i in 0..distance {
			lerp.lerp_hue(1.0);
			color = lerp.current_color;
		}
	}

	let _ = entities.build_entity()
		.with(Position::new(x as i32, y as i32), positions)
		.with(Character::new(character, color, colors::BLACK), characters)
		.with(Description::new(String::from("Grass"), String::from("A patch of tall, green cave grass.")), descriptions)
		.with(LightMask, light_masks)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();
}

fn make_small_mushroom (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>,  
				lights: &mut WriteStorage<Light>, descriptions: &mut WriteStorage<Description>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: i32, y: i32) {
	let color: Color;

	if number_generator.gen_range(0,100) < 50 {
		color = colors::CYAN;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color = colors::TURQUOISE;
		} else {
			color = colors::AZURE;
		}
	}

	let _ = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(Character::new('\u{2B}', color, colors::BLACK), characters)
		.with(Description::new(String::from("Mushroom"), String::from("A small, biolumescent fungus. It glows a erie blue.")), descriptions)
		.with(Light::new(2, color), lights)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();
}

fn make_large_mushroom (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>,  
				lights: &mut WriteStorage<Light>, descriptions: &mut WriteStorage<Description>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: i32, y: i32) {
	let character: char;
	let color: Color;

	if number_generator.gen_range(0, 100) < 80 {
		character = '\u{06}';
	} else {
		character = '\u{05}';
	}

	if number_generator.gen_range(0,100) < 50 {
		color = colors::CYAN;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color = colors::TURQUOISE;
		} else {
			color = colors::AZURE;
		}
	}

	let _ = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(Character::new(character, color, colors::BLACK), characters)
		.with(Description::new(String::from("Mushroom"), String::from("A large, biolumescent fungus. It glows a erie blue.")), descriptions)
		.with(Light::new(5, color), lights)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();
}

fn make_shallow_water (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>,
				descriptions: &mut WriteStorage<Description>, light_masks : &mut WriteStorage<LightMask>, color_lerps: &mut WriteStorage<ColorLerp>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: i32, y: i32) {
	let offset: f64 = number_generator.gen::<f64>() * 10.0;
	let rate: f64 = number_generator.gen::<f64>() * 10.0;

	let _ = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(Character::new('\u{F7}', colors::BLUE, colors::BLACK), characters)
		.with(ColorLerp::new(colors::BLUE, colors::LIGHTER_BLUE, rate, offset), color_lerps)
		.with(Description::new(String::from("Water"), String::from("Shimmers on the walls and ceiling. You can see bottom.")), descriptions)
		.with(LightMask, light_masks)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();
}

fn make_shallow_lava (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>, 
				lights: &mut WriteStorage<Light>, descriptions: &mut WriteStorage<Description>, color_lerps: &mut WriteStorage<ColorLerp>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: i32, y: i32) {
	let offset: f64 = number_generator.gen::<f64>() * 10.0;
	let rate: f64 = number_generator.gen::<f64>() * 1.75;

	let _ = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(Character::new('\u{F7}', colors::RED, colors::BLACK), characters)
		.with(ColorLerp::new(colors::RED, colors::GREY, rate, offset), color_lerps)
		.with(Description::new(String::from("Lava"), String::from("Eminates heat. Glows a dull red and gray.")), descriptions)
		.with(TileTag, tile_tags)
		.with(Light::new(5, colors::RED), lights)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.build();
}

fn make_deep_water (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>,
				descriptions: &mut WriteStorage<Description>, light_masks : &mut WriteStorage<LightMask>, color_lerps: &mut WriteStorage<ColorLerp>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: i32, y: i32) {
	let offset: f64 = number_generator.gen::<f64>() * 10.0;
	let rate: f64 = number_generator.gen::<f64>() * 10.0;

	let color_b: Color;

	if number_generator.gen_range(0,100) < 50 {
		color_b = colors::BLUE;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color_b = colors::DARKER_BLUE;
		} else {
			color_b = colors::LIGHT_BLUE;
		}
	}

	let _ = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(Character::new('\u{F7}', colors::DARK_BLUE, colors::BLACK), characters)
		.with(ColorLerp::new(colors::DARK_BLUE, color_b, rate, offset), color_lerps)
		.with(Description::new(String::from("Water"), String::from("Shimmers on the walls and ceiling. Darkness lies beneath the surface.")), descriptions)
		.with(LightMask, light_masks)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.with(TileTag, tile_tags)
		.build();
}

fn make_deep_lava (entities: &Entities, positions: &mut WriteStorage<Position>, characters: &mut WriteStorage<Character>, dirty_flags : &mut WriteStorage<DirtyFlag>, static_flags: &mut WriteStorage<StaticFlag>, 
				lights: &mut WriteStorage<Light>, descriptions: &mut WriteStorage<Description>, color_lerps: &mut WriteStorage<ColorLerp>, tile_tags: &mut WriteStorage<TileTag>, number_generator: &mut StdRng, x: i32, y: i32) {
	let offset: f64 = number_generator.gen::<f64>() * 10.0;
	let rate: f64 = number_generator.gen::<f64>() * 4.0;

	let color_b: Color;

	if number_generator.gen_range(0,100) < 50 {
		color_b = colors::YELLOW;
	} else {
		if number_generator.gen_range(0,100) < 50 {
			color_b = colors::RED;
		} else {
			color_b = colors::FLAME;
		}
	}
	let _ = entities.build_entity()
		.with(Position::new(x, y), positions)
		.with(Character::new('\u{F7}', colors::ORANGE, colors::BLACK), characters)
		.with(ColorLerp::new(colors::ORANGE, color_b, rate, offset), color_lerps)
		.with(Description::new(String::from("Lava"), String::from("Eminates strong heat. Glows a range of yellows, oranges, and reds.")), descriptions)
		.with(TileTag, tile_tags)
		.with(Light::new(10, colors::RED), lights)
		.with(StaticFlag, static_flags)
		.with(DirtyFlag, dirty_flags)
		.build();
}