use tcod::console::*;
use tcod::input::Key;
use tcod::colors;

use specs::{Dispatcher, World, Builder};

use crate::state::State;
use crate::state::StateAction;
use crate::state::WorldAction;
use crate::state::CurrentWorldAction;
use crate::application::{Context, DeltaTime};

use crate::description_state::DescriptionState;
use crate::level_transistion_state::LevelTransition;

use crate::components::basic::{Position, Character, CycleAnimation, ColorLerp, Description, Actor, Light};
use crate::components::tags::{PlayerTag, TileTag, LookCursorTag, DirtyFlag, StaticFlag};

use crate::systems::render;
use crate::level_generation::map::{LightMap, TransparencyMap, VisionMap};
use crate::systems::player::{PlayerControlSystem, CurrentInput, PlayerVisionSystem};
use crate::systems::actor::{ActorAction, ActorSystem};
use crate::systems::level::{SpawnPosition, ExitPosition, ClearLevelSystem, NewLevelSystem};

pub struct TestState <'a, 'b> {
	world: World,
	seed: String,
	current_level: i32,
	update_dispatcher: Dispatcher<'a, 'b>,
	new_level_dispatcher: Dispatcher<'a, 'b>,
	offscreen: Offscreen,
}

impl <'a, 'b> TestState <'a, 'b> {

	pub fn new (ctx: Context) -> TestState <'a, 'b> {
		// create world and register resources
		let mut world = World::new();
		world.register::<Position>();
		world.register::<Character>();
		world.register::<Description>();
		world.register::<Actor>();
		world.register::<CycleAnimation>();
		world.register::<ColorLerp>();
		world.register::<Light>();
		world.register::<PlayerTag>();
		world.register::<LookCursorTag>();
		world.register::<TileTag>();

		world.add_resource(CurrentInput(None));
		world.add_resource(CurrentWorldAction(WorldAction::default()));
		world.add_resource(DeltaTime(0.0));

		world.add_resource(VisionMap(Vec::default()));
		world.add_resource(LightMap(Vec::default()));
		world.add_resource(TransparencyMap(Vec::default()));

		world.add_resource(ExitPosition((0,0)));
		world.add_resource(SpawnPosition((0,0)));

		let seed = String::from("Test");

		// make dispatchers
		//update dispatcher is called every frame and updates the following systems
		let mut update_dispatcher = specs::DispatcherBuilder::new()
				.with(PlayerControlSystem {is_looking: false}, "player_controls", &[])
				.with(ActorSystem::new(), "action_system", &["player_controls"])
				.with(render::AnimationSystem, "animation_system", &[])
				.with(render::LightingSystem, "lighting_system", &["animation_system"])
				.with(PlayerVisionSystem, "vision_system", &["lighting_system"])
				.build();

		update_dispatcher.setup(&mut world.res);

		// new level dispatcher is called whenever a new level needs to be generated.
		let mut new_level_dispatcher = specs::DispatcherBuilder::new()
				.with(ClearLevelSystem, "clear_system", &[])
				.with(NewLevelSystem::new(seed.clone()), "new_level_system", &["clear_system"])
				.build();

		new_level_dispatcher.setup(&mut world.res);
		
		let offscreen = Offscreen::new(ctx.width, ctx.height);

		TestState {
			world: world,
			seed: seed.clone(),
			current_level: 1,
			update_dispatcher: update_dispatcher,
			new_level_dispatcher: new_level_dispatcher,
			offscreen: offscreen,
		}
	}

}

impl <'a, 'b> State for TestState <'a, 'b> {
	
	fn init (&mut self) {

		self.new_level_dispatcher.dispatch(&mut self.world.res);
		
		let spawn_position = self.world.read_resource::<SpawnPosition>().0;

		// player
		self.world.create_entity()
			.with(Position::new(spawn_position.0, spawn_position.1))
			.with(Character::new('@', colors::WHITE, colors::RED))
			.with(CycleAnimation::new(4.0, vec!['@', '!']))
			.with(Description::new(String::from("You"), String::from("Damn you ugly")))
			.with(Light::new(15))
			.with(Actor::new(ActorAction::WaitForInput))
			.with(PlayerTag)
			.build();

		// make light
		self.world.create_entity()
			.with(Position::new(50, 50))
			.with(Character::new('&', colors::YELLOW, colors::BLACK))
			.with(Light::new(20))
			.with(StaticFlag)
			.with(DirtyFlag)
			.build();
	}
	
	fn on_enter (&mut self) {
		let last_world_action = self.world.read_resource::<CurrentWorldAction>().0.clone();
		match last_world_action {
			WorldAction::GoToNewLevel => {
				self.new_level_dispatcher.dispatch(&mut self.world.res);
			},
			_ => {},
		}
		self.world.write_resource::<CurrentWorldAction>().0 = WorldAction::NoAction;
	}
	
	fn on_exit (&mut self) {
		
	}

	fn update (&mut self, _ctx: &Context, delta: DeltaTime, input: Option<Key>) -> StateAction {
		self.world.write_resource::<CurrentInput>().0 = input;

		self.world.write_resource::<DeltaTime>().0 = delta.0;
		self.update_dispatcher.dispatch(&mut self.world.res);
		self.world.maintain();

		match &self.world.read_resource::<CurrentWorldAction>().0 {
			WorldAction::Exit => {return StateAction::Exit},
			WorldAction::PushDescriptionState(name, desc, color) => {
				return StateAction::Push(Box::new(DescriptionState::new(name.clone(), desc.clone(), *color)));
			},
			WorldAction::GoToNewLevel => {self.current_level += 1; return StateAction::Push(Box::new(LevelTransition::new(self.current_level)))},
			WorldAction::NoAction => {return StateAction::NoAction},
			_ => {return StateAction::NoAction},
		}
	}

	fn render (&mut self, ctx: &Context, window: &mut Root) {
		// TODO make one render function that calls sub functions for entities and gui elements so that clear() blit() and flush() in one place
		self.offscreen.clear();
		render::render(ctx, &mut self.offscreen, window, & self.world);

		//? This is placed here for debugging purposes
		let mut seed_text = String::from("Seed: ");
		seed_text.push_str(&self.seed);
		window.print_ex(ctx.width/2, ctx.height-1, BackgroundFlag::None, TextAlignment::Center, seed_text);

	}
}