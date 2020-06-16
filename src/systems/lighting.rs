use bracket_lib::prelude::RGB;
use specs::{ReadStorage, WriteStorage, System, Read};

use crate::components::basic::{Light, Position, Renderable};
use lightmask::LightMask;
use crate::level_generation::map::Map;
use crate::systems::render::{ObjectShader};
use crate::components::tag::PlayerTag;

pub struct LightingSystem {
    player_coords : (i32,i32),
}

impl LightingSystem {
    pub fn new () -> Self {
        LightingSystem {
            player_coords : (0, 0),
        }
    }
}

impl<'a> System<'a> for LightingSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, PlayerTag>,
        WriteStorage<'a, Renderable>,
        ReadStorage<'a, Light>,
        Read<'a, Map>
    );

    fn run(&mut self, (positions, player_tag, mut renderables, lights, map): Self::SystemData) {
        use specs::Join;

        let mut light_mask = LightMask::new(map.width, map.height);
        for (position, _player) in (&positions, &player_tag).join() {
            self.player_coords = (position.x, position.y);
        }

        for (position, light) in (&positions, &lights).join() {
            //#! This is an important enough performance save to properly invest in it. Significantly less lights are rendered at a time
            if !((self.player_coords.0 - position.x).abs() >= 20) && !((self.player_coords.1 - position.y).abs() >= 20) {
                light_mask.add_light(&position, &light);
            }
        }
        light_mask.set_ambient(map.ambient_light);

        light_mask.compute_mask(&map.transparency_map);
  
        //apply shading to renderables
        for (position, renderable) in (&positions, &mut renderables).join() {
            if !(renderable.fg_shader == ObjectShader::NoShading && renderable.bg_shader == ObjectShader::NoShading){
                let x = position.x as usize;
                let y = position.y as usize;
                let r_br = light_mask.mask[x + y * light_mask.width].0 as f32;
                let g_br = light_mask.mask[x + y * light_mask.width].1 as f32;
                let b_br = light_mask.mask[x + y * light_mask.width].2 as f32;
                renderable.shading = RGB::from_f32(r_br, g_br, b_br);
            }
        }  
    }
}

pub mod lightmask {
    use crate::components::basic::{Position, Light};
    use super::lightmask_helper::{Node, compute_channel};
    use bracket_lib::prelude::RGB;

    #[derive(Debug)]
    pub struct LightMask {
        pub mask : Vec<(f32,f32,f32)>,
        pub width: usize,
        pub height: usize,
        ambient_light : RGB,
        distance_map: Vec<(f32,f32,f32)>,
        lights: Vec<Node>,
    }

    impl LightMask {
        pub fn new(width: usize, height: usize) -> Self {
            LightMask {
                mask: vec![(0.0, 0.0, 0.0); width * height],
                width: width,
                height: height,
                ambient_light : RGB::from_f32(0.0, 0.0, 0.0),
                distance_map: vec![(0.0, 0.0, 0.0); width * height],
                lights: Vec::new(),
            }
        }

        pub fn add_light(&mut self, position : &Position, light : &Light) {
            let x = position.x as usize;
            let y = position.y as usize;
            let rad = light.radius as f32;
            let cost_r = -1.0 * rad * light.color.r;
            let cost_g = -1.0 * rad * light.color.g;
            let cost_b = -1.0 * rad * light.color.b;
            self.distance_map[x + y * self.width] = (cost_r, cost_g, cost_b);
            self.lights.push(Node::new(cost_r, cost_g, cost_b, position));
        }

        pub fn set_ambient (&mut self, ambient : RGB) {
            self.ambient_light = ambient;
        }

        pub fn compute_mask (&mut self, walls : &Vec<f32>) {
            compute_channel(self.width, self.height, &self.lights, &mut self.distance_map, &mut self.mask, walls);

            for i in 0..self.mask.len() {
                self.mask[i].0 += self.ambient_light.r;
                self.mask[i].1 += self.ambient_light.g;
                self.mask[i].2 += self.ambient_light.b;
            }
        } 
    }
}

mod lightmask_helper {
    use crate::components::basic::{Position};

    const SOME_CONSTANT : f32 = 10.0;

    #[derive(Copy, Debug, Clone)]
    pub struct Node {
        pub r_cost : f32,
        pub g_cost : f32,
        pub b_cost : f32,
        pub pos : (i32,i32),
    }

    impl Node {
        pub fn new (r_cost : f32, g_cost : f32, b_cost : f32, position : &Position)-> Self {
            Node {
                r_cost : r_cost,
                g_cost : g_cost,
                b_cost : b_cost,
                pos : (position.x, position.y),
            }
        }

        pub fn get_total_cost (&self) -> f32 {
            return self.r_cost + self.g_cost + self.b_cost;
        }
    }
    pub fn compute_channel(width : usize, height : usize, lights : &Vec<Node>, distance_map : &mut Vec<(f32, f32, f32)>, mask : &mut Vec<(f32, f32, f32)>, transparency : &Vec<f32>,) {
        let mut priority_queue: Vec<Node> = Vec::new();
        
        //push all the lights to the queue
        for light_source in lights {
            priority_queue.push(light_source.clone());
            let x = light_source.pos.0 as usize;
            let y = light_source.pos.1 as usize;
            let r_cost = -1.0 * light_source.r_cost / SOME_CONSTANT;
            let g_cost = -1.0 * light_source.g_cost / SOME_CONSTANT;
            let b_cost = -1.0 * light_source.b_cost / SOME_CONSTANT;
            mask[x+y*width] = (r_cost, g_cost, b_cost);
        }

        while !priority_queue.is_empty() {
            let current_node = priority_queue.pop().unwrap();
            let x = current_node.pos.0;
            let y = current_node.pos.1;
            
            //for each neighbor
            for dx in x - 1..x + 2 {
                for dy in y - 1..y + 2 {
                    //make sure its in bounds and not the current node
                    if !(dx==x && dy==y) && !(dx >= width as i32 || dx < 0) && !(dy >= height as i32 || dy < 0) {
                        
                        //get distance to neighbor
                        let distance_to_neighbor = if ((x - dx).pow(2) + (y - dy).pow(2)) == 2 {
                            1.414
                        } else {
                            1.0
                        };

                        // calculate cost. This makes sure walls are lit but transparency still works.
                        let calculated_r_cost =
                            if transparency[x as usize + y as usize * width] > 0.0 {
                                (1.0 - transparency[x as usize + y as usize * width])
                                    * (current_node.r_cost + distance_to_neighbor)
                            } else {
                                current_node.r_cost + distance_to_neighbor
                            };

                        let calculated_g_cost =
                            if transparency[x as usize + y as usize * width] > 0.0 {
                                (1.0 - transparency[x as usize + y as usize * width])
                                    * (current_node.g_cost + distance_to_neighbor)
                            } else {
                                current_node.g_cost + distance_to_neighbor
                            };

                        let calculated_b_cost =
                            if transparency[x as usize + y as usize * width] > 0.0 {
                                (1.0 - transparency[x as usize + y as usize * width])
                                    * (current_node.b_cost + distance_to_neighbor)
                            } else {
                                current_node.b_cost + distance_to_neighbor
                            };

                        //make sure that the calculated cost is lower
                        let mut push_red_channel = false;
                        let mut push_green_channel = false;
                        let mut push_blue_channel = false;

                        if calculated_r_cost < distance_map[dx as usize + dy as usize * width].0 {
                            distance_map[dx as usize + dy as usize * width].0 = calculated_r_cost;
                            let red_value = -1.0 * calculated_r_cost / SOME_CONSTANT;
                            mask[dx as usize + dy as usize * width].0 = red_value;
                            push_red_channel = true;
                        }

                        if calculated_g_cost < distance_map[dx as usize + dy as usize * width].1 {
                            distance_map[dx as usize + dy as usize * width].1 = calculated_g_cost;
                            let green_value = -1.0 * calculated_g_cost / SOME_CONSTANT;
                            mask[dx as usize + dy as usize * width].1 = green_value;
                            push_green_channel = true;
                        }

                        if calculated_b_cost < distance_map[dx as usize + dy as usize * width].2 {
                            distance_map[dx as usize + dy as usize * width].2 = calculated_b_cost;
                            let blue_value = -1.0 * calculated_b_cost / SOME_CONSTANT;
                            mask[dx as usize + dy as usize * width].2 = blue_value;
                            push_blue_channel = true;
                        }

                        if push_red_channel || push_green_channel || push_blue_channel {
                            let mut neighbor_node = Node {
                                r_cost : 0.0,
                                g_cost : 0.0,
                                b_cost : 0.0,
                                pos : (dx,dy),
                            };

                            if push_red_channel {
                                neighbor_node.r_cost = calculated_r_cost;
                            }

                            if push_green_channel {
                                neighbor_node.g_cost = calculated_g_cost;
                            }

                            if push_blue_channel {
                                neighbor_node.b_cost = calculated_b_cost;
                            }

                            priority_queue.push(neighbor_node);
                            priority_queue.sort_by(|a, b| b.get_total_cost().partial_cmp(&a.get_total_cost()).unwrap());

                        }
                    }
                }
            }
        }
    }
}
