use specs::{World, WriteStorage, Read, System, ReadStorage, Entities, Write};

use tcod::console::*;
use tcod::map::Map as TcodMap;
use tcod::map::FovAlgorithm;

use crate::application::{DeltaTime};

use crate::components::basic::{Position, Character, CycleAnimation, ColorLerp, Light, LightMask};
use crate::components::tags::{PlayerTag, TileTag, LookCursorTag, DirtyFlag, StaticFlag};
use crate::level_generation::map::{LightMap, TransparencyMap, VisionMap};

//* Lighting branch

use crate::application::Context;



//TODO make this function only render entities
//? Might not need root to be passed into this function
pub fn render (ctx: &Context, offscreen: &mut Offscreen, window: &mut Root, world: &World) {
	use specs::Join;
	
	// Load storages
	let (positions, characters, player, tiles, look_cursor) = (world.read_storage::<Position>(), 
										   world.read_storage::<Character>(),
										   world.read_storage::<PlayerTag>(),
										   world.read_storage::<TileTag>(),
										   world.read_storage::<LookCursorTag>(),

										   );


	// Get player world coordinates
	let mut player_x = 0;
	let mut player_y = 0;
	for (position, _character, _player) in (&positions, &characters, &player).join() {
		player_x = position.x;
		player_y = position.y;
	}

	// Draw tile entities
	for (position, character, _tile) in (&positions, &characters, &tiles).join() {
		// convert from world coordinates to screen coordinates relative to player position.
		let screen_x = { (ctx.width / 2) + (position.x - player_x)};
		let screen_y = { (ctx.height /2) + (position.y - player_y)};

		//if the screen coordinates are within the bounds of the screen, draw the entity.
		if on_screen((screen_x, screen_y), (ctx.width, ctx.height)) {
			let vision_map : &Vec<Vec<bool>> = &world.read_resource::<VisionMap>().0;
			if vision_map[position.x as usize][position.y as usize] {
				offscreen.put_char_ex(screen_x, screen_y, character.glyph, character.current_foreground, character.current_background);
			}
			
		}
	}

	// Draw all other non-player, non-tile entities
	for (position, character, _player, _tile) in (&positions, &characters, !&player, !&tiles).join() {
		// convert from world coordinates to screen coordinates relative to player position.
		let screen_x = { (ctx.width / 2) + (position.x - player_x)};
		let screen_y = { (ctx.height /2) + (position.y - player_y)};

		//if the screen coordinates are within the bounds of the screen, draw the entity.
		if on_screen((screen_x, screen_y), (ctx.width, ctx.height)) {
			let vision_map : &Vec<Vec<bool>> = &world.read_resource::<VisionMap>().0;
			if vision_map[position.x as usize][position.y as usize] {
				offscreen.put_char_ex(screen_x, screen_y, character.glyph, character.current_foreground, character.current_background);
			}
		}
	}

	// Draw player
	for (position, character, _player) in (&positions, &characters, &player).join() {
		// convert from world coordinates to screen coordinates relative to player position.
		// In this case, the player is drawn in the middle of the window
		let screen_x = { (ctx.width / 2) + (position.x - player_x)};
		let screen_y = { (ctx.height /2) + (position.y - player_y)};

		//if the screen coordinates are within the bounds of the screen, draw the entity.
		if on_screen((screen_x, screen_y), (ctx.width, ctx.height)) {
			let light_map : &Vec<Vec<bool>> = &world.read_resource::<VisionMap>().0;
			if light_map[position.x as usize][position.y as usize] {
				offscreen.put_char_ex(screen_x, screen_y, character.glyph, character.current_foreground, character.current_background);
			}
		}
	}

	//Draw the look cursor if its there
	//this is done last because we want the cursor to be overtop everything else.
	for (position, character, _cursor) in (&positions, &characters, &look_cursor).join() {
		// convert from world coordinates to screen coordinates relative to player position.
		let screen_x = { (ctx.width / 2) + (position.x - player_x)};
		let screen_y = { (ctx.height /2) + (position.y - player_y)};

		//if the screen coordinates are within the bounds of the screen, draw the entity.
		if on_screen((screen_x, screen_y), (ctx.width, ctx.height)) {
			offscreen.put_char_ex(screen_x, screen_y, character.glyph, character.current_foreground, character.current_background);
		}
	}

	//blit everything from the offscreen to root
	blit(offscreen, (0, 0), (ctx.width, ctx.height), window, (0, 0), 1.0, 1.0);
}

fn on_screen (screen_coords:(i32,i32), screen_size: (i32,i32)) -> bool {
	let mut draw_entity = true;
	if ((screen_coords.0 < 0) || (screen_coords.0 > screen_size.0)) || ((screen_coords.1 < 0) || (screen_coords.1 > screen_size.1)) {
		draw_entity = false;
	}
	return draw_entity;
}

pub struct AnimationSystem;

impl<'a> System<'a> for AnimationSystem {

	type SystemData = (
		WriteStorage<'a, CycleAnimation>,
		WriteStorage<'a, ColorLerp>,
		WriteStorage<'a, Character>,
		Read<'a, DeltaTime>
	);

	fn run (&mut self, (mut cycles, mut lerps, mut characters, delta) : Self::SystemData) {
		use specs::Join;

		for (cycle, character) in (&mut cycles, &mut characters).join() {
			cycle.cycle(& delta.0);
			character.glyph = cycle.frames[cycle.current_frame];
		}

		for (lerp, character) in (&mut lerps, &mut characters).join() {
			lerp.lerp_hue(delta.0);

			character.current_foreground = lerp.current_color;
		}
	}

}

pub struct LightingSystem;

impl<'a> System<'a> for LightingSystem {
	type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, TileTag>,
		ReadStorage<'a, StaticFlag>,
		ReadStorage<'a, LightMask>,
		WriteStorage<'a, Light>,
		WriteStorage<'a, Character>,
		WriteStorage<'a, DirtyFlag>,
		
		Write<'a, LightMap>,
		Read<'a, TransparencyMap>,
		Entities<'a>,
	);

	fn run (&mut self, (positions, tile_tags, static_flags, light_masks, mut lights, mut characters, mut dirty_flags, mut light_maps, mut transparency_map, entities) : Self::SystemData) {
		use specs::Join;
		//TODO just work on static lights first
		let mut cleaned_lights : Vec<u32> = Vec::new();
		for (position, light, _dirty_flag, _static_flag, entity) in (&positions, &mut lights, &mut dirty_flags, &static_flags, &entities).join() {
			// get the width and height of the map		
			let width = transparency_map.0.len();
			let height = transparency_map.0[0].len();
			//make the fov map
			let mut this_light_map = TcodMap::new(width as i32, height as i32);
			
			// set the points on the fov map for transparency
			for x in 0..width as i32 {
				for y in 0..height as i32 {
					this_light_map.set(x, y, transparency_map.0[x as usize][y as usize], false);
				}
			}

			//compute the fov from the position of the light
			this_light_map.compute_fov(position.x as i32, position.y as i32, light.radius, true, FovAlgorithm::Shadow);

			//set the points on the map as lit
			for x in 0..width {
				for y in 0..height {
					if this_light_map.is_in_fov(x as i32, y as i32){
						let mut distance : f64 = ((position.x - x as i32).pow(2) + (position.y - y as i32).pow(2)) as f64;
						distance = distance.sqrt();
						let attentuation = light.get_attenuation(distance);
						if attentuation > light_maps.0[x][y] {
							light_maps.0[x][y] = attentuation;
						}
					}
				}
			}

			//add the light to the cleaned list
			cleaned_lights.push(entity.id());
		}

		//remove the dirty flag from the cleaned lights
		for light in cleaned_lights.iter() {
			dirty_flags.remove(entities.entity(*light));
		}

		let mut cleaned_entities : Vec<u32> = Vec::new();
		//change attentuation of static entities affected by light
		for (position, _light_mask, _dirty_flag, _static_flag, character, entity) in (&positions, &light_masks, &dirty_flags, &static_flags, &mut characters, &entities).join() {
			let char_color : tcod::Color = character.current_foreground;
			character.current_foreground = tcod::Color::new_from_hsv(char_color.hsv().0, char_color.hsv().1, light_maps.0[position.x as usize][position.y as usize] as f32);
			cleaned_entities.push(entity.id());
		}

		//remove the dirty flag from the cleaned entities
		for entity in cleaned_entities.iter() {
			dirty_flags.remove(entities.entity(*entity));
		}

	}
}