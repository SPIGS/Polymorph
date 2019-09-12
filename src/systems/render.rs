use specs::{World, WriteStorage, Read, System};

use tcod::console::*;

use crate::application::{DeltaTime};

use crate::components::basic::{Position, Character, CycleAnimation, ColorLerp};
use crate::components::tags::{PlayerTag, TileTag, LookCursorTag};
use crate::systems::light::{VisionMap};

//* Lighting branch

use crate::application::Context;
use std::ops::Add;

#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub struct ColorWithAlpha {
	pub rgb : tcod::Color,
	pub alpha : f64,
}

impl ColorWithAlpha {
	pub fn new (r : u8, g : u8, b : u8, a : f64) -> Self {
		Self {
			rgb : tcod::Color::new(r, g, b),
			alpha : a,
		}
	}

	pub fn new_from_tcod (color : tcod::Color , a : f64) -> Self {
		Self {
			rgb : color,
			alpha : a,
		}
	}	
}

impl Add<Color2> for Color2 {
	type Output = Color2;

	fn add (self, other : Color2) -> Color2 {
		let r = ((self.rgb.r as u16 + other.rgb.r as u16)/2) as u8;
		let g = ((self.rgb.g as u16 + other.rgb.g as u16)/2) as u8;
		let b = ((self.rgb.b as u16 + other.rgb.b as u16)/2) as u8;
		Self {
			rgb : tcod::Color::new(r, g, b),
			alpha : (self.alpha + other.alpha).min(1.0),
		}
	}
}

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