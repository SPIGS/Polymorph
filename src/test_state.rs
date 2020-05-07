use bracket_lib::prelude::BTerm;
use bracket_lib::prelude::VirtualKeyCode;
use bracket_lib::prelude::DrawBatch;
use bracket_lib::prelude::render_draw_buffer;
use bracket_lib::prelude::RGB;
use bracket_lib::prelude::Rect;

//use specs::{Dispatcher, World, Builder};
use specs::prelude::{World, WorldExt, Dispatcher, Builder};

use crate::state::{StateAction, State, CurrentInput, DeltaTime};
use crate::components::basic::{Position, Renderable, Inventory, Currency, Actor, Light};
use crate::components::tag::PlayerTag;
use crate::components::gui::{PlayerCard, Panel, Justification};

use crate::systems::render::{RenderSystem, GUIRenderSystem};
use crate::systems::actor::PlayerMoveSystem;
use crate::systems::player::PickUpSystem;
use crate::systems::gui::GUIUpdate;
use crate::systems::lighting::LightingSystem;

pub struct TestState <'a, 'b>{
    world : World,
    update_dispatcher : Dispatcher<'a, 'b>,
    render_dispatcher : Dispatcher<'a, 'b>,
    gui_render_dispatcher  : Dispatcher<'a, 'b>,
    screen_size : (u32,u32),
}

impl <'a, 'b> TestState <'a, 'b> {
    pub fn new (ctx : &mut BTerm) -> Self {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Renderable>();
        world.register::<Inventory>();
        world.register::<Currency>();
        world.register::<Actor>();
        world.register::<PlayerCard>();
        world.register::<Panel>();
        world.register::<Light>();

        world.insert(DeltaTime(0.0));
        world.insert(CurrentInput::default());
        
        let mut update_dispatcher = specs::DispatcherBuilder::new()
                .with(PlayerMoveSystem, "move_system", &[])
                .with(PickUpSystem, "pickup_system", &[])
                .with(GUIUpdate, "gui_update", &[])
                .build();
        update_dispatcher.setup(&mut world);

        let render_system = RenderSystem::new(DrawBatch::new(), ctx.get_char_size());
        let lighting_system = LightingSystem{};
        let gui_render_system = GUIRenderSystem::new(DrawBatch::new(), ctx.get_char_size());

        let mut render_dispatcher = specs::DispatcherBuilder::new()
                .with(lighting_system, "lighting_system", &[])
                .with(render_system, "render_system", &["lighting_system"])
                .build();
        render_dispatcher.setup(&mut world);

        let mut gui_render_dispatcher = specs::DispatcherBuilder::new()
                .with(gui_render_system, "gui_render_system", &[])
                .build();
        gui_render_dispatcher.setup(&mut world);

        TestState {
            world : world,
            update_dispatcher : update_dispatcher,
            render_dispatcher : render_dispatcher,
            gui_render_dispatcher : gui_render_dispatcher,
            screen_size : ctx.get_char_size(),
        }
    }
}

impl <'a, 'b> State for TestState <'a ,'b> {

    fn init (&mut self) {
       self.world.create_entity()
            .with(Position::new(0, 0))
            .with(PlayerTag)
            .with(Inventory::new())
            .with(Renderable::new(64, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), false))
            .with(Light::new(20, 1.0, RGB::from_f32(1.0, 1.0, 1.0)))
            .with(Actor::new())
            .build();

        for x in 0..80 {
            for y in 0..40 {
                self.world.create_entity()
                    .with(Position::new(x, y))
                    .with(Renderable::new(46, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), false))
                    .build();
            }
        }

        for x in 20..40 {
            for y in 10..30 {
                if x == 20 || y == 10 || y == 30 || x==40{
                    self.world.create_entity()
                        .with(Position::new(x, y))
                        .with(Renderable::new(219, RGB::from_f32(1.0, 0.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), false))
                        .build();
                }
            }
        }

        self.world.create_entity()
            .with(Position::new(3, 3))
            .with(Renderable::new(224, RGB::from_f32(1.0, 1.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), true))
            .with(Light::new(15, 1.0, RGB::from_f32(1.0, 0.0, 0.0)))
            .build();

        self.world.create_entity()
            .with(Position::new(79, 39))
            .with(Renderable::new(224, RGB::from_f32(1.0, 1.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), true))
            .with(Light::new(20, 1.0, RGB::from_f32(0.0, 0.0, 1.0)))
            .build();

        self.world.create_entity()
            .with(Position::new(79, 0))
            .with(Renderable::new(224, RGB::from_f32(1.0, 1.0, 0.0), RGB::from_f32(0.0, 0.0, 0.0), true))
            .with(Light::new(30, 1.0, RGB::from_f32(1.0, 1.0, 1.0)))
            .build();


        let player_card_bounds = Rect::with_size(0, 0, self.screen_size.0/4, self.screen_size.1);

        self.world.create_entity()
            .with(Panel::new(player_card_bounds, true, Justification::RIGHT, Option::None))
            .with(PlayerCard::new())
            .build();

        info!("Initialized state");
    }

    fn on_enter (&mut self) {}

    fn update (&mut self, ctx : &mut BTerm, input : CurrentInput, delta_time : DeltaTime) -> StateAction {
        {
        let mut delta = self.world.write_resource::<DeltaTime>();
        *delta = delta_time;
        }
        {
        let mut current_input = self.world.write_resource::<CurrentInput>();
        *current_input = input;
        }
        self.update_dispatcher.dispatch(&mut self.world);
        self.world.maintain();
        match ctx.key {
            None => {return StateAction::NoAction},
            Some(key) => {

                match key {
                    VirtualKeyCode::Escape => {return StateAction::Exit},
                    VirtualKeyCode::F2 => {
                        info!("Screenshot");
                        ctx.screenshot("screenshots/screenshot.png");
                        return StateAction::NoAction
                    },
                    _ => {return StateAction::NoAction},
                }
            }
        }
    }

    fn render (&mut self, ctx : &mut BTerm) {
      self.render_dispatcher.dispatch(&mut self.world);

      let draw_result = render_draw_buffer(ctx);
      match draw_result{
            Ok(_v) => {},
            Err(e) => {
                error!("Error on rendering draw buffer : {}", e);
            },
      }
      //this is done so the gui is rendered on top of everythign else
      self.gui_render_dispatcher.dispatch(&mut self.world);
      let draw_result = render_draw_buffer(ctx);
      match draw_result{
            Ok(_v) => {},
            Err(e) => {
                error!("Error on rendering draw buffer : {}", e);
            },
      }
    }
    
    fn on_exit (&mut self) {}
}