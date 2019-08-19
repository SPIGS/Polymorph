use tcod::console::*;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::*;
use crate::test_state::TestState;

#[derive(Clone)]
pub struct Context {
	pub width: i32,
	pub height: i32,
	pub title: String,
}

impl Context {
	pub fn builder () -> Context {
		Context {
			width: 80,
			height: 60,
			title: String::from("Default Title"),
		}
	}

	pub fn size (&mut self, screen_width: i32, screen_height: i32) -> &mut Context {
		self.width = screen_width;
		self.height = screen_height;
		return self;
	}

	pub fn title (&mut self, title: String) -> &mut Context {
		self.title = title;
		return self;
	}

	pub fn build (&self) -> Context {
		Context {
			width: self.width,
			height: self.height,
			title: self.title.clone(),
		}
	}
}

pub struct DeltaTime (pub f64);

impl Default for DeltaTime {
	fn default() -> Self {DeltaTime(0.0)}
}

pub struct Application {
	root: Root,
	ctx: Context,
	manager: StateManager,
	is_running: bool,
	max_fps: i32,
}

impl Application {
	pub fn new (screen_width: i32, screen_height: i32, title: String, fps_limit: i32) -> Application {

		let app_context = Context::builder()
			.size(screen_width, screen_height)
			.title(title)
			.build();

		let root_console = Root::initializer()
			.font("assets/terminal-big.png", FontLayout::AsciiInRow)
			.size(app_context.width, app_context.height)
			.font_type(FontType::Greyscale)
			.title(app_context.title.clone())
			.init();

		let mut state_manager = StateManager::new();
		state_manager.push(Box::new(TestState::new(app_context.clone())));

		Application {
			root: root_console,
			ctx: app_context,
			manager: state_manager,
			is_running: false,
			max_fps: fps_limit,
		}
	}

	pub fn run (&mut self) {
		use crate::input::KeyFlags;

		self.is_running = true;
		
		let mut last_time = get_current_time_nano();
		let nanosecs_per_tick = 1000000000.0 / self.max_fps as f64;
		let mut delta : f64 = 0.0;
		let mut timer = get_current_time_millis();
		let mut frames = 0;
		let mut ticks = 0;
		let mut should_render: bool;

		while self.is_running {
			let now = get_current_time_nano();
			delta += (now - last_time) as f64 / nanosecs_per_tick;
			last_time = now;
			should_render = false;

			while delta >= 1.0 {
				ticks += 1;
				let key = self.root.check_for_keypress(KeyFlags::key_pressed());
				self.manager.update_current_state(&self.ctx, DeltaTime(delta), key);
				delta -= 1.0;
				should_render = true;
			}

			if should_render {
				self.root.clear();
				
				self.manager.render_current_state(& self.ctx, &mut self.root);
				
				self.root.flush();
				frames += 1;
			}
			

			if get_current_time_millis() - timer > 1000 {
				timer += 1000;
				println!("FPS: {}, Ticks: {}", frames, ticks);
				frames = 0;
				ticks = 0;
			}

			if self.root.window_closed() {
					self.is_running = false;
			}
		}
	}
}

/// Gets the current time in milliseconds from the epoch.
pub fn get_current_time_millis () -> u128 {
	let start = SystemTime::now();
	let since_epoch = start.duration_since(UNIX_EPOCH)
		.expect("Contact Einstein.");
	return since_epoch.as_millis();
}

/// Gets the current time in nanoseconds from the epoch.
pub fn get_current_time_nano () -> u128 {
	let start = SystemTime::now();
	let since_epoch = start.duration_since(UNIX_EPOCH)
		.expect("Contact Einstein");
	return since_epoch.as_nanos();
}