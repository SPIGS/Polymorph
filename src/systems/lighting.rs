use bracket_lib::prelude::ColorPair;
use bracket_lib::prelude::Point;
use bracket_lib::prelude::Rect;
use bracket_lib::prelude::RGB;
use specs::{ReadStorage, WriteStorage, System};

use crate::components::basic::{Light, Position, Renderable};
use lightmask::LightMask;

pub struct LightingSystem;

impl<'a> System<'a> for LightingSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        ReadStorage<'a, Light>,
    );

    fn run(&mut self, (positions, mut renderables, lights): Self::SystemData) {
        use specs::Join;

        let mut light_mask = LightMask::new(80, 40);

        for (position, light) in (&positions, &lights).join() {
            light_mask.add_light(position.x as usize, position.y as usize, light.radius);
        }

        let mut walls = vec![0.0; 80*40];

        for x in 20..40 {
            for y in 10..30 {
                if x == 20 || y == 10 || y == 30 || x==40 {
                    walls[x+y*80] = 1.0;
                }
            }
        }
        
        light_mask.compute_mask(walls);

        for (position, renderable) in (&positions, &mut renderables).join() {
            if !renderable.shadeless {
                let x = position.x as usize;
                let y = position.y as usize;
                let br = light_mask.mask[x + y * light_mask.width] as f32;
                renderable.shading = (br,br,br);
            }
        }
    }
}

//TODO make good lighting

pub mod lightmask {
    use super::lightmask_helper::{Node, compute_node_distances};

    const SOME_CONSTANT : f64 = 15.0;

    #[derive(Debug)]
    pub struct LightMask {
        pub mask: Vec<f64>,
        pub width: usize,
        pub height: usize,
        distance_map: Vec<f64>,
        lights: Vec<Node>,
    }

    impl LightMask {
        pub fn new(width: usize, height: usize) -> Self {
            LightMask {
                mask: vec![0.0; width * height],
                width: width,
                height: height,
                distance_map: vec![0.0; width * height],
                lights: Vec::new(),
            }
        }

        pub fn add_light(&mut self, light_x: usize, light_y: usize, rad: u32) {
            self.distance_map[light_x + light_y * self.width] = -1.0 * rad as f64;
            self.lights.push(Node {
                cost: -1.0 * rad as f64,
                pos: (light_x as i32, light_y as i32),
            });
        }

        pub fn compute_mask(&mut self, transparency : Vec<f64>) {

            compute_node_distances(self.width, self.height, &self.lights, &mut self.distance_map, &transparency);

            for x in 0..self.width {
                for y in 0..self.height {
                    let distance = self.distance_map[x as usize + y as usize * self.width];
                    let br_mult = -1.0 * distance as f64 / SOME_CONSTANT;
                    let br = 1.0 * br_mult;
                    self.mask[x as usize + y as usize * self.width] = br;
                }
            }
        }
    }
}

mod lightmask_helper {

    #[derive(Copy, Debug, Clone)]
    pub struct Node {
        pub cost: f64,
        pub pos: (i32, i32),
    }

    pub fn compute_node_distances(
        width: usize,
        height: usize,
        lights: &Vec<Node>,
        distance_map: &mut Vec<f64>,
        transparency : &Vec<f64>,
    ) {
        let mut priority_queue: Vec<Node> = Vec::new();
        //push all the lights to the queue
        for light_source in lights {
            priority_queue.push(*light_source);
        }

        while !priority_queue.is_empty() {
            let current_node = priority_queue.pop().unwrap();
            let x = current_node.pos.0;
            let y = current_node.pos.1;

            //for each neighbor
            for dx in x - 1..x + 2 {
                for dy in y - 1..y + 2 {
                    //make sure its in bounds and not the current node
                    if dx + dy != 0
                        && !(dx >= width as i32 || dx < 0)
                        && !(dy >= height as i32 || dy < 0)
                    {
                        //get distance to neighbor
                        let mut distance_to_neighbor = ((x - dx).pow(2) + (y - dy).pow(2)) as f64;
                        distance_to_neighbor = distance_to_neighbor.sqrt();

                        // calculate cost. This makes sure walls are lit but transparency still works.
                        let calculated_cost =
                            if transparency[x as usize + y as usize * width] > 0.0 {
                                (1.0 - transparency[x as usize + y as usize * width])
                                    * (current_node.cost + distance_to_neighbor)
                            } else {
                                current_node.cost + distance_to_neighbor
                            };

                        if calculated_cost
                            < distance_map[dx as usize + dy as usize * width]
                        {
                            distance_map[dx as usize + dy as usize * width] =
                                calculated_cost;
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
