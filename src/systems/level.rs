use specs::{WriteStorage, System, Entities, Read};
use crate::level_generation::map::{Map, MapType};
use crate::level_generation::map::tile::TileType;
use crate::components::basic::{Position, Renderable, Light, ColorLerp};
use bracket_lib::prelude::RGB;
use crate::systems::render::ObjectShader;
use rand::{StdRng, SeedableRng, Rng};
pub struct LevelGenSystem;

impl <'a> System<'a> for LevelGenSystem {
    type SystemData = (
        WriteStorage <'a, Position>,
        WriteStorage <'a, Renderable>,
        WriteStorage <'a, Light>,
        WriteStorage <'a, ColorLerp>,
        Read<'a, Map>,
        Entities<'a>,
    );

    fn run (&mut self, (mut positions, mut renderables, mut lights, mut colorlerps, map, entities) : Self::SystemData) {
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
                            let offset: f32 = rng.gen::<f32>() * 1000.0;
                            let rate: f32 = rng.gen_range::<f32>(0.5, 1.0) * 3000.0;
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(0.6, 0.6, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .with(ColorLerp::new(RGB::from_u8(100, 100, 255), RGB::from_u8(175, 175, 255), rate, offset), &mut colorlerps)
                                    .build();
                        },
                        TileType::DeepWater => {
                            let offset: f32 = rng.gen::<f32>() * 1000.0;
                            let rate: f32 = rng.gen_range::<f32>(0.5, 1.0) * 3000.0;
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(0.0, 0.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .with(ColorLerp::new(RGB::from_u8(165, 165, 245), RGB::from_u8(0, 0, 200), rate, offset), &mut colorlerps)
                                    .build();
                        },
                        TileType::ShallowLava => {
                            let offset: f32 = rng.gen::<f32>() * 2000.0;
                            let rate: f32 = rng.gen_range::<f32>(0.5, 1.0) * 7500.0;
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(1.0, 0.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::NoShading, ObjectShader::Background), &mut renderables)
                                    .with(Light::new(6, 1.0, RGB::from_f32(1.0, 0.0, 0.0)), &mut lights)
                                    .with(ColorLerp::new(RGB::from_u8(255, 0, 0), RGB::from_u8(105, 105, 105), rate, offset), &mut colorlerps)
                                    .build();
                        },
                        TileType::DeepLava => {
                            let offset: f32 = rng.gen::<f32>() * 2000.0;
                            let rate: f32 = rng.gen_range::<f32>(0.5, 1.0) * 5000.0;
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(247, RGB::from_f32(1.0, 0.5, 0.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::NoShading, ObjectShader::Background), &mut renderables)
                                    .with(Light::new(6, 1.0, RGB::from_f32(1.0, 0.5, 0.0)), &mut lights)
                                    .with(ColorLerp::new(RGB::from_u8(255, 0, 0), RGB::from_u8(255, 175, 0), rate, offset), &mut colorlerps)
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
                        TileType::ThinWebs => {
                            make_web(&entities, &mut positions, &mut renderables, false, x, y);
                        },
                        TileType::ThickWebs => {
                            make_web(&entities, &mut positions, &mut renderables, true, x, y);
                        },
                        TileType::EggSac => {
                            make_egg_sac(&entities, &mut positions, &mut renderables, x, y);
                        },
                        TileType::TentTopCenter => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(196, RGB::from_u8(191, 151, 96), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::TentTopLeft => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(218, RGB::from_u8(191, 151, 96), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::TentTopRight => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(191, RGB::from_u8(191, 151, 96), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::TentBottomCenter => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(205, RGB::from_u8(191, 151, 96), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::TentBottomLeft => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(198, RGB::from_u8(191, 151, 96), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::TentBottomRight => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(181, RGB::from_u8(191, 151, 96), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::CampSeat => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(61, RGB::from_u8(145, 119, 61), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                        },
                        TileType::Fire => {
                            let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(30, RGB::from_u8( 245, 176, 65), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::NoShading, ObjectShader::Background), &mut renderables)
                                    .with(Light::new(10, 1.0, RGB::from_u8( 245, 176, 65)), &mut lights)
                                    .build();
                        },
                        TileType::HiveWall => {
                            let is_front_wall = if y == map.height-1 {
                                    false
                                } else {
                                    map.tiles[x+(y+1) * map.width] != TileType::HiveWall && map.tiles[x+(y+1) * map.width] != TileType::Empty
                                };
                            
                            if is_front_wall {
                                let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(223, RGB::from_u8(247, 220, 111), RGB::from_u8(243, 156, 18), ObjectShader::Foreground, ObjectShader::Foreground), &mut renderables)
                                    .build();
                            } else {
                                let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(219, RGB::from_u8(247, 220, 111), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                            }
                        },
                         TileType::HiveFloor => {
                            let _ = entities.build_entity()
                                .with(Position::new(x as i32, y as i32), &mut positions)
                                .with(Renderable::new_from_char('.', RGB::from_u8(243, 156, 18), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                .build();
                        },
                        _ => {},
                    }
                }
            }

    }
}

fn make_grass (entities: &Entities, positions: &mut WriteStorage<Position>, renderables: &mut WriteStorage<Renderable>, tall : bool, rng : &mut StdRng, x: usize, y: usize, distance: i32) {
    use bracket_lib::prelude::RgbLerp;

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
        if rng.gen_range(0, 100) < 70 {
            244
        } else {
            245
        } 
    } else {
        253
    };

	let _ = entities.build_entity()
		.with(Position::new(x as i32, y as i32), positions)
		.with(Renderable::new(char_id, color, RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), renderables)
		.build();
}

fn make_mushroom (entities: &Entities, positions: &mut WriteStorage<Position>, renderables: &mut WriteStorage<Renderable>, lights : &mut WriteStorage<Light>, large : bool, rng : &mut StdRng, x: usize, y: usize) {
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

fn make_web (entities: &Entities, positions: &mut WriteStorage<Position>, renderables: &mut WriteStorage<Renderable>, thick : bool, x: usize, y: usize) {
    let glyph = if thick {
        176
    } else {
        15
    };

    let _ = entities.build_entity()
		.with(Position::new(x as i32, y as i32), positions)
		.with(Renderable::new(glyph, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), renderables)
		.build();

}

fn make_egg_sac (entities: &Entities, positions: &mut WriteStorage<Position>, renderables: &mut WriteStorage<Renderable>, x: usize, y: usize) {
    let glyph = 7;
    let _ = entities.build_entity()
		.with(Position::new(x as i32, y as i32), positions)
		.with(Renderable::new(glyph, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), renderables)
		.build();

}
