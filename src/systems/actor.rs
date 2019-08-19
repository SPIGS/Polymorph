use specs::{ReadStorage, WriteStorage, System};

use crate::components::basic::{Position, Actor, DrunkWalkAI};

pub enum ActorAction {
	ActionWait,
	ActionMoveLeft,
	ActionMoveRight,
	ActionMoveUp,
	ActionMoveDown,
	ActionMoveUpLeft,
	ActionMoveUpRight,
	ActionMoveDownLeft,
	ActionMoveDownRight,
	ActionNoTime,
	WaitForInput, 	
}

pub struct ActorSystem{
	_actors: Vec<(Position, Actor)>
}

impl ActorSystem {
	pub fn new () -> Self {
		ActorSystem {
			_actors: Vec::new(),
		}
	}
}

impl<'a> System<'a> for ActorSystem {

	type SystemData = (
			WriteStorage<'a, Position>,
			ReadStorage<'a, Actor>,
		);

	fn run (&mut self, (mut positions, actors): Self::SystemData) {
		use specs::Join;
		for (position, actor) in (&mut positions, &actors).join() {
			match actor.action {
				ActorAction::ActionMoveLeft => {position.x -= 1;},
				ActorAction::ActionMoveRight => {position.x += 1},
				ActorAction::ActionMoveUp => {position.y -= 1},
				ActorAction::ActionMoveDown => {position.y += 1},
				ActorAction::ActionMoveUpLeft => {position.y -=1; position.x -= 1},
				ActorAction::ActionMoveUpRight => {position.y -=1; position.x += 1},
				ActorAction::ActionMoveDownLeft => {position.y +=1; position.x -= 1},
				ActorAction::ActionMoveDownRight => {position.y +=1; position.x += 1},
				_ => {},
			}
		}
	}
}

//TODO
pub struct AISystem {

}
//TODO
impl<'a> System<'a> for AISystem {
	type SystemData = (
			WriteStorage<'a, Actor>,
			ReadStorage<'a, DrunkWalkAI>,
		);

	fn run (&mut self, (mut actors, _drunkwalk_ai): Self::SystemData) {
		use specs::Join;
		use rand::Rng;
		for (actor, _drunk) in (&mut actors, &_drunkwalk_ai).join() {
			let mut rng = rand::thread_rng();
			let direction = rng.gen_range(0,4);

			if direction == 0 {
				actor.action = ActorAction::ActionMoveUp;
			} else if direction == 1 {
				actor.action = ActorAction::ActionMoveDown;
			} else if direction == 2 {
				actor.action = ActorAction::ActionMoveLeft;
			} else if direction == 3 {
				actor.action = ActorAction::ActionMoveRight;
			}
		}
	}
}