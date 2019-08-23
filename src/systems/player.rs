use specs::{ReadStorage, WriteStorage, System, Read, Entities, Write};

use tcod::colors;
use tcod::map::Map as TcodMap;
use tcod::map::FovAlgorithm;
use tcod::input::Key;
use tcod::input::KeyCode;

use crate::components::basic::{Position, Character, CycleAnimation, Description, Actor, Light};
use crate::components::tags::{PlayerTag, LookCursorTag, TileTag};

use crate::systems::actor::ActorAction;

use crate::state::{CurrentWorldAction, WorldAction};
use crate::systems::level::ExitPosition;

use crate::level_generation::map::{LightMap, VisionMap, TransparencyMap};


#[derive(Default)]
pub struct CurrentInput (pub Option<Key>);

pub struct PlayerControlSystem {
	pub is_looking: bool,
}

impl<'a> System<'a> for PlayerControlSystem {

	type SystemData = (
			WriteStorage<'a, Position>, 
			WriteStorage<'a, Character>,
			WriteStorage<'a, Actor>,
			WriteStorage<'a, CycleAnimation>,
			ReadStorage<'a, Description>,
			ReadStorage<'a, PlayerTag>,
			ReadStorage<'a, TileTag>,
			WriteStorage<'a, LookCursorTag>,
			Read<'a, CurrentInput>,
			Read<'a, ExitPosition>,
			Write<'a, CurrentWorldAction>,
			Entities<'a>,
		);

	fn run (&mut self, (mut positions, mut characters, mut actors, mut cycle_animations, descriptions, 
						player_tags, tile_tags, mut look_cursor, current_input, exit_position, mut current_world_action, entities): Self::SystemData) {
		use specs::Join;

		if !self.is_looking {
			// get normal player input
			for (position, actor, _player_tag) in (&mut positions, &mut actors, &player_tags).join() {
				match current_input.0 {
					// arrow keys
					Some(Key{code: KeyCode::Up, ..}) => {actor.action = ActorAction::ActionMoveUp},
					Some(Key{code: KeyCode::Down, ..}) => {actor.action = ActorAction::ActionMoveDown},
					Some(Key{code: KeyCode::Left, ..}) => {actor.action = ActorAction::ActionMoveLeft},
					Some(Key{code: KeyCode::Right, ..}) => {actor.action = ActorAction::ActionMoveRight},

					// numpad keys
					Some(Key{code: KeyCode::NumPad8, ..}) => {actor.action = ActorAction::ActionMoveUp},
					Some(Key{code: KeyCode::NumPad2, ..}) => {actor.action = ActorAction::ActionMoveDown},
					Some(Key{code: KeyCode::NumPad4, ..}) => {actor.action = ActorAction::ActionMoveLeft},
					Some(Key{code: KeyCode::NumPad6, ..}) => {actor.action = ActorAction::ActionMoveRight},
					Some(Key{code: KeyCode::NumPad7, ..}) => {actor.action = ActorAction::ActionMoveUpLeft},
					Some(Key{code: KeyCode::NumPad9, ..}) => {actor.action = ActorAction::ActionMoveUpRight},
					Some(Key{code: KeyCode::NumPad1, ..}) => {actor.action = ActorAction::ActionMoveDownLeft},
					Some(Key{code: KeyCode::NumPad3, ..}) => {actor.action = ActorAction::ActionMoveDownRight},
					

					Some(Key{code: _, printable: 'l', ..}) => {
						self.is_looking = true;
						actor.action = ActorAction::WaitForInput;
					},

					Some(Key{code: _, printable: '.', shift: true, ..}) => {
						let exit_point = exit_position.0;
						if position.x == exit_point.0 && position.y == exit_point.1 {
							current_world_action.0 = WorldAction::GoToNewLevel;
							actor.action = ActorAction::WaitForInput;
						} else {
							actor.action = ActorAction::WaitForInput;
						}
					},
					_ => {actor.action = ActorAction::WaitForInput},
				}	
			}

			if self.is_looking {

				let mut player_x = 0;
				let mut player_y = 0;
				// have the cursor's starting position be the players current position.
				for (position, _player_tag) in (&positions, &player_tags).join() {
					player_x = position.x;
					player_y = position.y;
				}
				
				// create the cursor entity
				let cursor = entities.create();
				let _ = look_cursor.insert(cursor, LookCursorTag);
				let _ = positions.insert(cursor, Position::new(player_x, player_y));
				let _ = characters.insert(cursor, Character::new('&', colors::YELLOW, colors::BLACK));
				let _ = cycle_animations.insert(cursor, CycleAnimation::new(5.0, vec!['X', ' ']));
			}

		} else {

			let mut show_description = false;
			let mut look_position = (0,0);
			for (position, _look_cursor_tag, e) in (&mut positions, &look_cursor, &entities).join() {
				// get the input for the cursor
				match current_input.0 {
					// arrow keys
					Some(Key{code: KeyCode::Up, ..}) => {position.y -= 1},
					Some(Key{code: KeyCode::Down, ..}) => {position.y += 1},
					Some(Key{code: KeyCode::Left, ..}) => {position.x -= 1},
					Some(Key{code: KeyCode::Right, ..}) => {position.x += 1},

					// numpad keys
					Some(Key{code: KeyCode::NumPad8, ..}) => {position.y -= 1},
					Some(Key{code: KeyCode::NumPad2, ..}) => {position.y += 1},
					Some(Key{code: KeyCode::NumPad4, ..}) => {position.x -= 1},
					Some(Key{code: KeyCode::NumPad6, ..}) => {position.x += 1},
					Some(Key{code: KeyCode::NumPad7, ..}) => {position.y -= 1; position.x -= 1},
					Some(Key{code: KeyCode::NumPad9, ..}) => {position.y -= 1; position.x += 1},
					Some(Key{code: KeyCode::NumPad1, ..}) => {position.y += 1; position.x -= 1},
					Some(Key{code: KeyCode::NumPad3, ..}) => {position.y += 1; position.x += 1},

					// escape - exits look mode
					Some(Key{code: KeyCode::Escape, ..}) => {
						self.is_looking = false;
						let _ = entities.delete(e);
					},
					// enter - shows name of tile and descirption in new state.
					Some(Key{code: KeyCode::Enter, ..}) => {
						show_description = true;
						look_position = (position.x, position.y);
					},
					_ => {},
				}
			}

			if show_description {
				let mut not_tile_found = false;
				//First iterate over the non tile entities on the cursor position. The player would want to look at that instead of the ground.
				for (position, _look_cursor_tag, description, character, _tile_tag) in (&positions, !&look_cursor, &descriptions, &characters, !&tile_tags).join() {
					if position.x == look_position.0 && position.y == look_position.1 {
						current_world_action.0 = WorldAction::PushDescriptionState(description.name.clone(), description.description.clone(), character.default_foreground);
						
						// set not_tile_found to true because we found an entity that wasn't a tile and we dont want to push two states to the stack.
						not_tile_found = true;
					}
				}

				// if we did not find a non-tile entity we want to get the description of the map tile instead.
				if !not_tile_found {
					for (position, _look_cursor_tag, description, character) in (&positions, !&look_cursor, &descriptions, &characters).join() {
						if position.x == look_position.0 && position.y == look_position.1 {
							current_world_action.0 = WorldAction::PushDescriptionState(description.name.clone(), description.description.clone(), character.default_foreground);
						}
					}
				}
			}
		}		
	}
}

pub struct PlayerVisionSystem;

impl<'a> System<'a> for PlayerVisionSystem {

	type SystemData = (
		ReadStorage<'a, Position>,
		ReadStorage<'a, PlayerTag>,
		WriteStorage<'a, Light>,
		Read<'a, LightMap>,
		Write<'a, VisionMap>,
		Read <'a, TransparencyMap>
	);

	fn run (&mut self, (positions, player_tags, mut lights, light_maps, mut vision_map, transparency_map) : Self::SystemData) {
		use specs::Join;
		for (position, light, _player_tag) in (&positions, &mut lights, &player_tags).join() {
			if position.x != light.position.0 || position.y != light.position.1 {
				// update light position
				light.position.0 = position.x;
				light.position.1 = position.y;
				
				// get width and height and reset vision map
				let width = vision_map.0.len();
				let height = vision_map.0[0].len();
				vision_map.0 = vec![vec![false; height]; width];

				// maek new tcod structure for the map
				let mut new_vision_map = TcodMap::new(width as i32, height as i32);

				// set the transparency
				for x in 0..width as i32 {
					for y in 0..height as i32 {
						new_vision_map.set(x,y, transparency_map.0[x as usize][y as usize], false);
					}
				}
				
				//compute the fov at double the radius so as to include possible light sources in the distance. 
				new_vision_map.compute_fov(position.x as i32, position.y as i32, light.radius * 2, true, FovAlgorithm::Shadow);
				
				// set the tiles in the players actual visiual radius to visible
				for x in -light.radius..light.radius {
					for y in -light.radius..light.radius {
						if x*x + y*y <= light.radius*light.radius {
							let dig_x = position.x + x;
							let dig_y = position.y + y;
							if dig_x >= 0 && dig_x < width as i32 && dig_y >= 0 && dig_y < height as i32 {
								vision_map.0[dig_x as usize][dig_y as usize] = new_vision_map.is_in_fov(dig_x, dig_y);
							} 
						}
					}
				}

				// set the light sources in the distance that are within line of sight to be visible 
				for x in 0..width {
					for y in 0..height {
						if light_maps.0[x][y] > 0.0 && new_vision_map.is_in_fov(x as i32, y as i32) {
							vision_map.0[x][y] = true;
						}
					}
				}
			}
		}
	}
}