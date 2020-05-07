use specs::{WriteStorage, System, Entities, Read};
use crate::level_generation::map::{Map, MapType, TileType};
use crate::components::basic::{Position, Renderable};
use bracket_lib::prelude::RGB;
use crate::systems::render::ObjectShader;
pub struct LevelGenSystem;

impl <'a> System<'a> for LevelGenSystem {
    type SystemData = (
        WriteStorage <'a, Position>,
        WriteStorage <'a, Renderable>,
        Read<'a, Map>,
        Entities<'a>,
    );

    fn run (&mut self, (mut positions, mut renderables, map, entities) : Self::SystemData) {
        if map.map_type == MapType::Cavern {
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
                                    map.tiles[x+(y+1) * map.width] == TileType::Floor
                                };
                            
                            if is_front_wall {
                                let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(223, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.75, 0.75, 0.75), ObjectShader::Foreground, ObjectShader::Foreground), &mut renderables)
                                    .build();
                            } else {
                                let _ = entities.build_entity()
                                    .with(Position::new(x as i32, y as i32), &mut positions)
                                    .with(Renderable::new(219, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background), &mut renderables)
                                    .build();
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
    }
}
