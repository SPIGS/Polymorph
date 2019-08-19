use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode;
use tcod::colors;
use tcod::Color;

use crate::state::State;
use crate::state::StateAction;
use crate::application::{Context, DeltaTime};

pub struct DescriptionState {
	object_name: String,
	object_description: String,
	object_color: Color,
}

impl DescriptionState {
	pub fn new (name: String, description: String, color: Color) -> Self {
		DescriptionState {
			object_name: name,
			object_description: description,
			object_color: color,
		}
	}
}

impl State for DescriptionState {
	fn init (&mut self) {}

	fn on_enter(&mut self) {}

	fn on_exit(&mut self) {}

	fn update(&mut self, _ctx: &Context, _delta: DeltaTime, input: Option<Key>) -> StateAction {
		match input {
			Some(Key{code: KeyCode::Escape, ..}) => {return StateAction::Pop},
			_ => {return StateAction::NoAction},
		}
	}

	fn render(&mut self, ctx: &Context, window: &mut Root) {
		//print the object name
		window.print_ex(1, 1, BackgroundFlag::None, TextAlignment::Left, self.object_name.clone());

		// change the object name to the color of the object for aesthetics
		for x in 1..self.object_name.len()+1 {
			window.set_char_foreground(x as i32, 1, self.object_color);
		}
		
		let mut temp_desc = self.object_description.clone();

		//TODO make better
		// wrap the lines of the description
		if self.object_description.len() > ctx.width as usize {
			let mut last_space = 0;
			for (indice, character) in temp_desc.char_indices() {
				if (character == ' ') && (indice < ctx.width as usize) && (indice > last_space) {
					last_space = indice;
				} else if indice > ctx.width as usize{
					break
				}
			}
			temp_desc.insert(last_space + 1, '\n');
		}

		// print the lines of the description
		let mut line_offset = 0;
		for line in temp_desc.lines() {
			window.print_ex(1, 3 + line_offset, BackgroundFlag::None, TextAlignment::Left, line);
			line_offset += 1;
		}

		//print some gui and color it
		window.print_ex(1, ctx.height - 2, BackgroundFlag::None, TextAlignment::Left, String::from("Press Esc to continue..."));
		for x in 7..10 {
			window.set_char_foreground(x, ctx.height - 2, colors::GREEN);
		}
	}
}