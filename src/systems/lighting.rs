use bracket_lib::prelude::RGB;
use specs::{ReadStorage, WriteStorage, System, Read};

use crate::components::basic::{Light, Position, Renderable};
use lightmask::LightMask;
use crate::level_generation::map::Map;
use crate::systems::render::ObjectShader;

pub struct LightingSystem;

impl<'a> System<'a> for LightingSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        ReadStorage<'a, Light>,
        Read<'a, Map>
    );

    fn run(&mut self, (positions, mut renderables, lights, map): Self::SystemData) {
        use specs::Join;

        let mut light_mask = LightMask::new(map.width, map.height);

        for (position, light) in (&positions, &lights).join() {
            light_mask.add_light(&position, &light);
        }
        light_mask.set_ambient(RGB::from_f32(0.0, 0.0, 0.2));

        light_mask.compute_mask(&map.transparency_map);

        //apply shading to renderables
        for (position, renderable) in (&positions, &mut renderables).join() {
            if !(renderable.fg_shader == ObjectShader::NoShading && renderable.bg_shader == ObjectShader::NoShading){
                let x = position.x as usize;
                let y = position.y as usize;
                let r_br = light_mask.r_mask[x + y * light_mask.width] as f32;
                let g_br = light_mask.g_mask[x + y * light_mask.width] as f32;
                let b_br = light_mask.b_mask[x + y * light_mask.width] as f32;
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
        pub r_mask: Vec<f32>,
        pub g_mask: Vec<f32>,
        pub b_mask: Vec<f32>,
        pub width: usize,
        pub height: usize,
        ambient_light : RGB,
        distance_map_r: Vec<f32>,
        distance_map_g: Vec<f32>,
        distance_map_b: Vec<f32>,
        lights_r: Vec<Node>,
        lights_g: Vec<Node>,
        lights_b: Vec<Node>,
    }

    impl LightMask {
        pub fn new(width: usize, height: usize) -> Self {
            LightMask {
                r_mask: vec![0.0; width * height],
                g_mask: vec![0.0; width * height],
                b_mask: vec![0.0; width * height],
                width: width,
                height: height,
                ambient_light : RGB::from_f32(0.0, 0.0, 0.0),
                distance_map_r: vec![0.0; width * height],
                distance_map_g: vec![0.0; width * height],
                distance_map_b: vec![0.0; width * height],
                lights_r: Vec::new(),
                lights_g: Vec::new(),
                lights_b: Vec::new(),
            }
        }

        pub fn add_light(&mut self, position : &Position, light : &Light) {
            let x = position.x as usize;
            let y = position.y as usize;
            let rad = light.radius as f32;
            let cost_r = -1.0 * rad * light.color.r;
            let cost_g = -1.0 * rad * light.color.g;
            let cost_b = -1.0 * rad * light.color.b;
            self.distance_map_r[x + y * self.width] = cost_r;
            self.distance_map_g[x + y * self.width] = cost_g;
            self.distance_map_b[x + y * self.width] = cost_b;
            self.lights_r.push(Node::new(cost_r, position));
            self.lights_g.push(Node::new(cost_g, position));
            self.lights_b.push(Node::new(cost_b, position));
        }

        pub fn set_ambient (&mut self, ambient : RGB) {
            self.ambient_light = ambient;
        }

        pub fn compute_mask (&mut self, walls : &Vec<f32>) {
            compute_channel(self.width, self.height, &self.lights_r, &mut self.distance_map_r, &mut self.r_mask, walls);
            compute_channel(self.width, self.height, &self.lights_g, &mut self.distance_map_g, &mut self.g_mask, walls);
            compute_channel(self.width, self.height, &self.lights_b, &mut self.distance_map_b, &mut self.b_mask, walls);

            for i in 0..self.r_mask.len() {
                self.r_mask[i] += self.ambient_light.r;
            }
            for i in 0..self.g_mask.len() {
                self.g_mask[i] += self.ambient_light.g;
            }
            for i in 0..self.b_mask.len() {
                self.b_mask[i] += self.ambient_light.b;
            }
        } 
    }
}

mod lightmask_helper {
    use crate::components::basic::{Position};

    const SOME_CONSTANT : f32 = 10.0;

    #[derive(Copy, Debug, Clone)]
    pub struct Node {
        pub cost : f32,
        pub pos : (i32,i32),
    }

    impl Node {
        pub fn new (cost : f32, position : &Position)-> Self {
            Node {
                cost : cost,
                pos : (position.x, position.y),
            }
        }
    }
    pub fn compute_channel(width : usize, height : usize, lights : &Vec<Node>, distance_map : &mut Vec<f32>, mask : &mut Vec<f32>, transparency : &Vec<f32>,) {
        let mut priority_queue: Vec<Node> = Vec::new();
        
        //push all the lights to the queue
        for light_source in lights {
            priority_queue.push(light_source.clone());
            let x = light_source.pos.0 as usize;
            let y = light_source.pos.1 as usize;
            mask[x+y*width] = -1.0 * light_source.cost / SOME_CONSTANT;
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
                        let calculated_cost =
                            if transparency[x as usize + y as usize * width] > 0.0 {
                                (1.0 - transparency[x as usize + y as usize * width])
                                    * (current_node.cost + distance_to_neighbor)
                            } else {
                                current_node.cost + distance_to_neighbor
                            };

                        //make sure that the calculated cost is lower
                        if calculated_cost < distance_map[dx as usize + dy as usize * width] {
                            distance_map[dx as usize + dy as usize * width] = calculated_cost;
                            let br = -1.0 * calculated_cost / SOME_CONSTANT;
                            //add node to the brightness map
                            mask[dx as usize + dy as usize * width] = br;
                                
                            let neighbor_node = Node {
                                cost: calculated_cost,
                                pos: (dx, dy),
                            };

                            priority_queue.push(neighbor_node);
                            priority_queue.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap());
                        }
                    }
                }
            }
        }
    }
}
