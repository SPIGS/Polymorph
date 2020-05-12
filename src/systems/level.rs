use specs::{WriteStorage, System, Entities, Read};
use crate::level_generation::map::{Map, MapType};
use crate::level_generation::map::tile::TileType;
use crate::components::basic::{Position, Renderable, Light};
use bracket_lib::prelude::RGB;
use crate::systems::render::ObjectShader;
use rand::{StdRng, SeedableRng};
pub struct LevelGenSystem;

impl <'a> System<'a> for LevelGenSystem {
    type SystemData = (
        WriteStorage <'a, Position>,
        WriteStorage <'a, Renderable>,
        WriteStorage <'a, Light>,
        Read<'a, Map>,
        Entities<'a>,
    );

    fn run (&mut self, (mut positions, mut renderables, mut lights, map, entities) : Self::SystemData) {
        if map.map_type == MapType::Cavern {
            let mut rng : StdRng = SeedableRng::from_seed(map.hashed_seed.to_256_bit());
            for x in 0..map.width {
                for y in 0..map.height {
                    match map.tiles[x+y*map.width] {
                        TileType::Floor => {
                            let _ = entities.build_entity()
                                .with(Position::new(x as i32, y as i32), &mut positions)
                                .with(Renderable::new_from_char('.', RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                .build();
                        },
                        TileType::Wall => {
                            let is_front_wall = if y == map.height-1 {
                                    false
                                } else {
                                    map.tiles[x+(y+1) * map.width] != TileType::Wall && map.tiles[x+(y+1) * map.width] != TileType::Empty
                                };
                            
                            if is_front_wall {
                                let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(223, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.5, 0.5, 0.5), ObjectShader::Foreground, ObjectShader::Foreground), &mut renderables)
                                    .build();
                            } else {
                                let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(219, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                            }
                        },
                        TileType::ShallowWater => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(0.6, 0.6, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::DeepWater => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(0.0, 0.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::ShallowLava => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(1.0, 0.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::NoShading, ObjectShader::Background), &mut renderables)
                                    .with(Light::new(6, 1.0, RGB::from_f32(1.0, 0.0, 0.0)), &mut lights)
                                    .build();
                        },
                        TileType::DeepLava => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(1.0, 0.5, 0.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::NoShading, ObjectShader::Background), &mut renderables)
                                    .with(Light::new(6, 1.0, RGB::from_f32(1.0, 0.5, 0.0)), &mut lights)
                                    .build();
                        },
                        TileType::ShortGrass(d) => {
                            make_grass(&entities, &mut positions, &mut renderables, false, &mut rng, x, y, d);
                        },
                        TileType::TallGrass(d) => {
                            make_grass(&entities, &mut positions, &mut renderables, true, &mut rng, x, y, d);
                        },
                        TileType::SmallMushroom => {
                            make_mushroom(&entities, &mut positions, &mut renderables, &mut lights, false, &mut rng, x, y);
                        },
                        TileType::LargeMushroom => {
                             make_mushroom(&entities, &mut positions, &mut renderables, &mut lights, true, &mut rng, x, y);
                        },
                        _ => {},
                    }
                }
            }
        }
    }
}

fn make_grass (entities: &Entities, positions: &mut WriteStorage<Position>, renderables: &mut WriteStorage<Renderable>, tall : bool, rng : &mut StdRng, x: usize, y: usize, distance: i32) {
    use bracket_lib::prelude::RgbLerp;
    use rand::Rng;

    let color : RGB;

	// set the healthy color
	let color_healthy : RGB;
	if rng.gen_range(0,100) < 50 {
		color_healthy = RGB::from_u8(0, 255, 0);
	} else {
		if rng.gen_range(0,100) < 50 {
            color_healthy = RGB::from_u8(0, 168, 100);
		} else {
            color_healthy = RGB::from_u8(80, 180, 0);
		}
	}

	//set the unhealthy color
	let color_unhealthy : RGB;
	if rng.gen_range(0,100) < 50 {
		color_unhealthy = RGB::from_u8(191, 151, 96);
	} else {
		if rng.gen_range(0,100) < 50 {
            color_unhealthy = RGB::from_u8(145, 119, 61);
		} else {
            color_unhealthy = RGB::from_u8(207, 186, 81);
		}
	}

	let mut lerp = RgbLerp::new(color_healthy, color_unhealthy, 10);

	if distance == 0 {
		color = color_unhealthy;
	} else {
		let n = if distance >= 10 {
            9
        } else {
            distance as usize
        };
        color = lerp.nth(n).unwrap();
    }
    
    let char_id = if tall {
        244
    } else {
        34
    };

	let _ = entities.build_entity()
		.with(Position::new(x as i32, y as i32), positions)
		.with(Renderable::new(char_id, color, RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), renderables)
		.build();
}

fn make_mushroom (entities: &Entities, positions: &mut WriteStorage<Position>, renderables: &mut WriteStorage<Renderable>, lights : &mut WriteStorage<Light>, large : bool, rng : &mut StdRng, x: usize, y: usize) {
    use rand::Rng;
    let color: RGB;

    let character = if large {
        if rng.gen_range(0, 100) < 80 {
            6
        } else {
            5
        }
    } else {
        43
    };

    let light_rad = if large {
        8
    } else {
        4
    };

	if rng.gen_range(0,100) < 50 {
		color = RGB::from_u8(0, 255, 255);
	} else {
		if rng.gen_range(0,100) < 50 {
			color = RGB::from_u8(0, 255, 191);
		} else {
			color = RGB::from_u8(0, 127, 255);
		}
	}

	let _ = entities.build_entity()
		.with(Position::new(x as i32, y as i32), positions)
		.with(Renderable::new(character, color, RGB::from_u8(0, 0, 0), ObjectShader::NoShading, ObjectShader::Background), renderables)
		.with(Light::new(light_rad, 1.0, color), lights)
		.build();
}
