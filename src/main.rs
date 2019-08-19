extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate tcod;

pub mod state;
pub mod application;
pub mod input;
pub mod systems;
pub mod test_state;
pub mod description_state;
pub mod level_transistion_state;
pub mod components;
pub mod level_generation;

use application::Application;

fn main() {
	let mut app = Application::new(60, 40, String::from("Test Application"), 60);
	app.run();
}
