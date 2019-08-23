use tcod::colors::Color;

use specs::{Component, VecStorage, NullStorage};

use crate::systems::actor::*;

#[derive(Debug, Component, PartialEq)]
#[storage(VecStorage)]
pub struct Position {
	pub x: i32,
	pub y: i32,
}

impl Position {
	pub fn new (x: i32, y: i32) -> Self {
		Position {
			x: x,
			y: y,
		}
	}
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Actor {
	pub action: ActorAction,
	pub has_taken_turn: bool,
	pub on_turn: bool,
}

impl Actor {
	pub fn new (starting_action: ActorAction) -> Self {
		Actor {
			action: starting_action,
			has_taken_turn: false,
			on_turn: false,
		}
	}
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct DrunkWalkAI; 

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Character {
	pub glyph: char,
	pub default_foreground: Color,
	pub default_background: Color,
	pub current_foreground: Color,
	pub current_background: Color,
}

impl Character {
	pub fn new (glyph: char, foreground: Color, background: Color) -> Self {
		Character {
			glyph: glyph,
			default_foreground: foreground,
			default_background: background,
			current_foreground: foreground,
			current_background: background,
		}
	}
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Description {
	pub name: String,
	pub description: String,
}

impl Description {
	pub fn new (name: String, description: String) -> Self {

		Description {
			name: name,
			description: description,
		}
	}
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct CycleAnimation {
	pub rate_per_second: f64,
	pub current_frame: usize,
	pub frames: Vec<char>,
	time_to_change: f64,
	accumulator: f64,
}

impl CycleAnimation {

	pub fn new (rate_per_second: f64, frames: Vec<char>) -> CycleAnimation {

		let time_to_change = 100.0 / rate_per_second;

		CycleAnimation {
			rate_per_second: rate_per_second,
			current_frame: 0,
			frames: frames,
			time_to_change: time_to_change,
			accumulator: 0.0,
		}
	}
	
	pub fn cycle (&mut self, delta: &f64) {
		self.accumulator += delta;
		if self.accumulator >= self.time_to_change {
			
			self.current_frame += 1;
			if self.current_frame >= self.frames.len() {
				self.current_frame = 0;
			}
			self.accumulator = 0.0;
		}
	}
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct ColorLerp {
	pub current_color: Color,
	pub current_brightness : f32,
	color_a: Color,
	color_a_brightness : f32,
	color_b: Color,
	color_b_brightness : f32,
	time_to_step: f64,
	pub step: f64,
	step_accumulator : f64,
	accumulator: f64,
	add_step: bool,
}

impl ColorLerp {

	pub fn new (color_a: Color, color_b: Color, rate: f64, offset: f64) -> ColorLerp {

		let time_to_step = 100.0 / rate;

		ColorLerp {
			current_color: color_a,
			current_brightness : color_a.hsv().2,
			color_a: color_a,
			color_a_brightness : color_a.hsv().2,
			color_b: color_b,
			color_b_brightness : color_b.hsv().2,
			time_to_step: time_to_step,
			step: 0.1,
			step_accumulator: 0.0,
			accumulator: offset,
			add_step: true,
		}
	}

	pub fn lerp_hue (&mut self, delta: f64) {
		self.accumulator += delta;
		if self.accumulator >= self.time_to_step {

			let r = self.interpolate_channel(self.color_a.r as f64, self.color_b.r as f64);
			let g = self.interpolate_channel(self.color_a.g as f64, self.color_b.g as f64);
			let b = self.interpolate_channel(self.color_a.b as f64, self.color_b.b as f64);
			self.current_color = Color::new(r, g, b);

			if self.add_step && self.step_accumulator < 1.0 {
				self.step_accumulator += self.step;
				if self.step_accumulator >= 1.0 {
					self.step_accumulator = 1.0;
					self.add_step = !self.add_step;
				}
			} else if !self.add_step && self.step_accumulator > 0.0 {
				self.step_accumulator -= self.step;
				if self.step_accumulator <= 0.0 {
					self.step_accumulator = 0.0;
					self.add_step = !self.add_step;
				}
			}
			self.accumulator = 0.0;
		}
	}

	pub fn lerp_value (&mut self, delta : f64) {
		self.accumulator += delta;
		if self.accumulator >= self.time_to_step {
			let v = self.interpolate_value(self.color_a_brightness as f64, self.color_b_brightness as f64);
			self.current_brightness = v as f32;
			if self.add_step && self.step_accumulator < 1.0 {
				self.step_accumulator += self.step;
				if self.step_accumulator >= 1.0 {
					self.step_accumulator = 1.0;
					self.add_step = !self.add_step;
				}
			} else if !self.add_step && self.step_accumulator > 0.0 {
				self.step_accumulator -= self.step;
				if self.step_accumulator <= 0.0 {
					self.step_accumulator = 0.0;
					self.add_step = !self.add_step;
				}
			}
			self.accumulator = 0.0;
		}
	}

	fn interpolate_channel (&mut self, channel_a: f64, channel_b: f64) -> u8 {
		return ((channel_b - channel_a) * self.step_accumulator + channel_a) as u8; 
	}

	fn interpolate_value (&mut self, channel_a: f64, channel_b: f64) -> f64 {
		return (channel_b - channel_a) * self.step_accumulator + channel_a; 
	}

	fn reset (&mut self) {
		self.current_color = self.color_a;
		self.current_brightness = self.color_a_brightness;
		self.step_accumulator = 0.0;
		self.accumulator = 0.0;
	}

}

#[derive(Component, Default)]
#[storage(VecStorage)]
pub struct LightMask;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Light {
	pub radius : i32,
	pub position : (i32,i32),
}

impl Light {
	pub fn new (r: i32) -> Self {
		Light {
			radius : r,
			position : (0,0),
		}
	}

	/// Returns a value from 0.0 to 1.0 for attenuation. 1.0 being completely light, 0.0 being completely dark. 
	pub fn get_attenuation (&self, distance_from_light: f64) -> f64 {
		let a : f64 = 0.0;
		let b : f64 = 1.0 - (distance_from_light * distance_from_light) / (self.radius * self.radius) as f64;
		let mut attenuation = a.max(b);
		attenuation *= attenuation;
		return attenuation;
	}

}