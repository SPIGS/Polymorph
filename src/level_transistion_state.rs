use tcod::console::*;
use tcod::input::Key;

use crate::state::State;
use crate::state::StateAction;
use crate::application::{Context, DeltaTime};
use crate::application::get_current_time_millis;

const TRANSITION_TIME: u128 = 3000;

pub struct LevelTransition {
	prompt: String,
	last_time: u128,
}

impl LevelTransition {
	pub fn new (current_level: i32) -> Self {

		LevelTransition {
			prompt: format!("{}{}", "Level ", current_level),
			last_time: 0,
		}
	}
}

impl State for LevelTransition {
	fn init (&mut self) {
		self.last_time = get_current_time_millis();
	}

	fn on_enter(&mut self) {}

	fn on_exit(&mut self) {}

	fn update(&mut self, _ctx: &Context, _delta: DeltaTime, _input: Option<Key>) -> StateAction {
		let current_time = get_current_time_millis();
		if (current_time - self.last_time) >= TRANSITION_TIME {
			return StateAction::Pop;
		} else {
			return StateAction::NoAction;
		}
	}

	fn render(&mut self, ctx: &Context, window: &mut Root) {
		window.clear();
		window.print_ex(ctx.width/2, ctx.height/2, BackgroundFlag::None, TextAlignment::Center, &self.prompt);
	}
}