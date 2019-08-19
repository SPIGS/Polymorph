use std::collections::VecDeque;
use tcod::input::Key;
use tcod::console::Root;
use tcod::Color;

use crate::application::{Context, DeltaTime};

#[derive(Clone)]
pub enum WorldAction {
	NoAction,
	PushDescriptionState(String, String, Color),
	GoToNewLevel,
	Exit,

	Seeding,
	Digging,
	Cleaning,
	Smoothing,
	Watering,
	Planting,
	Growing,
}

impl Default for WorldAction {
	fn default() -> Self {WorldAction::NoAction}
}

#[derive(Default, Clone)]
pub struct CurrentWorldAction (pub WorldAction);

pub enum StateAction {
	NoAction,
	Push(Box<dyn State>),
	Pop,
	PopAmount(i32),
	Switch(Box<dyn State>),
	Exit,
}

pub trait State {
	/// Called when this state is pushed to the stack.
	fn init (&mut self);
	/// Called when this state becomes the top of the stack.
	fn on_enter (&mut self);
	/// Called when this state is popped or switched.
	fn on_exit (&mut self);
	/// Called routinely 60 times a second.
	fn update (&mut self, ctx: &Context, delta: DeltaTime, input: Option<Key>) -> StateAction;
	/// Called rountinely at a max of 60 times a second.
	fn render (&mut self, ctx: &Context, window: &mut Root);
}	

pub struct StateManager {
	pub states: VecDeque<Box<dyn State>>, 
}

impl StateManager {

	pub fn new () -> StateManager {
		StateManager {
			states: VecDeque::new(),
		}
	}

	pub fn push (&mut self, state: Box<dyn State>) {
		self.states.push_front(state);
		self.states[0].init();
	}

	pub fn pop (&mut self) {
		self.states[0].on_exit();
		self.states.pop_front();
		self.states[0].on_enter();
	}

	pub fn pop_amount (&mut self, amt: i32) {
		for _i in 0..amt {
			self.pop();
		}
	}

	pub fn switch (&mut self, state: Box<dyn State>){
		self.pop();
		self.push(state);
	}

	pub fn exit (&mut self) {
		let length = self.states.len();

		for _i in 0..length {
			self.states.pop_front();
		}
	}

	pub fn update_current_state (&mut self, ctx: &Context, delta: DeltaTime, input: Option<Key>) {
		let action = self.states[0].update(ctx, delta, input);

		match action {
			StateAction::Pop => self.pop(),
			StateAction::PopAmount(number) => self.pop_amount(number),
			StateAction::Push(new_state) => self.push(new_state),
			StateAction::Switch(new_state) => self.switch(new_state),
			StateAction::Exit => self.exit(),
			StateAction::NoAction => {},
		}
	}

	pub fn render_current_state (&mut self, ctx: &Context, window: &mut Root) {
		self.states[0].render(ctx, window);
	}
}