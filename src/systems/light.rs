use specs::{WriteStorage, Read, System, ReadStorage, Entities, Write};

use tcod::map::Map as FOVMap;
use tcod::map::FovAlgorithm;
use tcod::{Color, colors};

use crate::systems::render::ColorWithAlpha;

use crate::components::basic::{Position, Character, Light, LightMask};
use crate::components::tags::{DirtyFlag, StaticFlag};

#[derive(Default)]
pub struct VisionMap (pub Vec<Vec<bool>>);

#[derive(Default)]
pub struct DynamicLightMap (pub Vec<Vec<ColorWithAlpha>>);

#[derive(Default)]
pub struct StaticLightMap (pub Vec<Vec<ColorWithAlpha>>);

#[derive(Default)]
pub struct TransparencyMap (pub Vec<Vec<bool>>);

pub struct BakedLightingSystem;

impl<'a> System<'a> for BakedLightingSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, StaticFlag>,
        WriteStorage<'a, Light>,
        WriteStorage<'a, DirtyFlag>,

        Write<'a, StaticLightMap>,
        Read<'a, TransparencyMap>,
        Entities<'a>,
    );


    fn run (&mut self, (positions, static_flags, mut lights, mut dirty_flags, mut light_maps, transparency_map, entities) : Self::SystemData) {
        use specs::Join;

		let mut cleaned_lights : Vec<u32> = Vec::new();
		for (position, light, _dirty_flag, _static_flag, entity) in (&positions, &mut lights, &mut dirty_flags, &static_flags, &entities).join() {
			// get the width and height of the map		
			let width = transparency_map.0.len();
			let height = transparency_map.0[0].len();
			//make the fov map
			let mut this_light_map = FOVMap::new(width as i32, height as i32);
			
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
                        let light_color = ColorWithAlpha::new_from_tcod(light.color, attentuation);
					    light_maps.0[x][y] = light_maps.0[x][y] + light_color;
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
	}
}

pub struct DynamicLightSystem;

impl<'a> System <'a> for DynamicLightSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, StaticFlag>,
        WriteStorage<'a, Light>,
        WriteStorage<'a, DirtyFlag>,

        Write<'a, DynamicLightMap>,
        Read<'a, TransparencyMap>,
        Entities<'a>,
    );

    fn run (&mut self, (positions, static_flags, mut lights, mut dirty_flags, mut light_map, transparency_map, entities) : Self::SystemData) {
        use specs::Join;
        let mut dirty_lights : Vec<u32> = Vec::new();
        let mut changed = false;
        //update the lights' positions
        for (position, light, _static_flag, _dirty_flag, entity) in (&positions, &mut lights, !&static_flags, !&dirty_flags, &entities).join() {
            // if the light's position changed, update it and put a dirty flag on it
            if (position.x != light.position.0 || position.y != light.position.1) && !changed {
                changed = true;
                light.position.0 = position.x;
                light.position.1 = position.y;
                let width = light_map.0.len();
                let height = light_map.0[0].len();
                light_map.0 = vec![vec![ColorWithAlpha::new_from_tcod(colors::BLACK, 0.0); height]; width];
                dirty_lights.push(entity.id());
            } else if changed {
                light.position.0 = position.x;
                light.position.1 = position.y;
                dirty_lights.push(entity.id());
            }
        }

        //put dirty flags on the lights
        for light in dirty_lights.iter() {
            let _ = dirty_flags.insert(entities.entity(*light), DirtyFlag);
        }

        // clean and recalculate dynamic lights
        let mut cleaned_lights : Vec<u32> = Vec::new();

        for (light, _static_flag, _dirty_flag, entity) in (&mut lights, !&static_flags, &dirty_flags, &entities).join() {
            let width = transparency_map.0.len();
            let height = transparency_map.0[0].len();

            let mut this_light_map = FOVMap::new(width as i32, height as i32);

            for x in 0..width as i32 {
                for y in 0..height as i32 {
                    this_light_map.set(x, y, transparency_map.0[x as usize][y as usize], false);
                }
            }

            this_light_map.compute_fov(light.position.0 as i32, light.position.1 as i32, light.radius, true, FovAlgorithm::Shadow);

            for x in 0..width {
                for y in 0..height {
                    if this_light_map.is_in_fov(x as i32, y as i32) {
                        let mut distance : f64 = ((light.position.0 - x as i32).pow(2) + (light.position.1 - y as i32).pow(2)) as f64;
                        distance = distance.sqrt();
                        let attentuation = light.get_attenuation(distance);
                        let light_color = ColorWithAlpha::new_from_tcod(light.color, attentuation);
					    light_map.0[x][y] = light_map.0[x][y] + light_color;
                    }
                }
            }
            cleaned_lights.push(entity.id());
        }

        for light in cleaned_lights.iter() {
            dirty_flags.remove(entities.entity(*light));
        }
    }
}

///Combines the colors and alphas from the lightmaps and whatever is being lit and updates the color.
pub struct LightCombine;

impl <'a> System <'a> for LightCombine {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, LightMask>,
        WriteStorage<'a, Character>,

        Read<'a, StaticLightMap>,
        Read<'a, DynamicLightMap>,
    );

    fn run (&mut self, (positions, light_masks, mut characters, static_light, dynamic_light) : Self::SystemData) {
        use specs::Join;
        for (position, _light_mask, character) in (&positions, &light_masks, &mut characters).join() {
            let char_foreground_color = ColorWithAlpha::new_from_tcod(character.default_foreground, 0.0);
			let mixed_foreground_color = char_foreground_color + static_light.0[position.x as usize][position.y as usize] + dynamic_light.0[position.x as usize][position.y as usize];
            character.current_foreground = Color::new_from_hsv(mixed_foreground_color.rgb.hsv().0, mixed_foreground_color.rgb.hsv().1, mixed_foreground_color.alpha as f32);
        }
    }
}