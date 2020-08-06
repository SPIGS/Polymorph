use bracket_lib::prelude::BTerm;
use bracket_lib::prelude::VirtualKeyCode;
use bracket_lib::prelude::DrawBatch;
use bracket_lib::prelude::render_draw_buffer;
use bracket_lib::prelude::RGB;
use bracket_lib::prelude::Rect;

//use specs::{Dispatcher, World, Builder};
use specs::prelude::{World, WorldExt, Dispatcher, Builder};

use crate::state::{StateAction, State, PortableContext, make_portable_ctx, WorldState, CurrentWorldState};
use crate::components::basic::{Position, Renderable, Inventory, Currency, Actor, Light, ColorLerp, CycleAnimation, LightFlicker};
use crate::components::tag::PlayerTag;
use crate::components::gui::{PlayerCard, Panel, HorizontalAlignment,VerticalAlignment, PanelBuilder, TextBoxBuilder, TextBox, DebugInfoBox};

use crate::systems::render::{RenderSystem, GUIRenderSystem};
use crate::systems::actor::{PlayerMoveSystem, VisibilitySystem};
use crate::systems::player::PickUpSystem;
use crate::systems::gui::GUIUpdate;
use crate::systems::lighting::LightingSystem;
use crate::level_generation::map::{Map, MapType, VisibilityMap};
use crate::systems::level::LevelGenSystem;
use crate::systems::render::ObjectShader;
use crate::systems::animation::AnimationSystem;

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
        world.register::<ColorLerp>();
        world.register::<CycleAnimation>();
        world.register::<LightFlicker>();
        world.register::<TextBox>();
        world.register::<DebugInfoBox>();

        world.insert(PortableContext::default());
        world.insert(CurrentWorldState(WorldState::NoAction));

        let seed = String::from("adsfasds");
        let mut map = Map::new(100, 100, seed, MapType::Cavern, RGB::from_f32(0.0, 0.0, 0.0));
        map.generate();
        world.insert(map);

        world.insert(VisibilityMap::new(100, 100));
        
        let mut level_gen_dispatcher = specs::DispatcherBuilder::new()
                .with(LevelGenSystem, "level_gen", &[])
                .build();
        level_gen_dispatcher.setup(&mut world);

        level_gen_dispatcher.dispatch(&world);

        let mut update_dispatcher = specs::DispatcherBuilder::new()
                .with(PlayerMoveSystem, "move_system", &[])
                .with(PickUpSystem, "pickup_system", &[])
                .with(GUIUpdate, "gui_update", &[])
                .with(AnimationSystem, "animation_update", &[])
                .build();
        update_dispatcher.setup(&mut world);

        let render_system = RenderSystem::new(DrawBatch::new(), ctx.get_char_size());
        let lighting_system = LightingSystem::new();
        let gui_render_system = GUIRenderSystem::new(DrawBatch::new(), ctx.get_char_size());
        let visibility_system = VisibilitySystem{};

        let mut render_dispatcher = specs::DispatcherBuilder::new()
                .with(visibility_system, "visibility", &[])
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

    fn init (&mut self, ctx : &mut BTerm) {
        //create player
        self.world.create_entity()
            .with(Position::new(0, 0))
            .with(PlayerTag)
            .with(Inventory::new())
            .with(Renderable::new(64, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), ObjectShader::Foreground, ObjectShader::Background))
            .with(Light::new(10, 1.0, RGB::from_f32(0.75, 0.53, 0.0)))
            .with(Actor::new())
            .build();

        // let panel = PanelBuilder::new()
        //                     .width_percentage(25)
        //                     .height_percentage(100)
        //                     .with_horiz_align(HorizontalAlignment::RIGHT)
        //                     .with_vert_align(VerticalAlignment::CENTER)
        //                     .is_decorated(true)
        //                     .title(String::from("Title"))
        //                     .title_color(RGB::from_u8(255, 0, 0))
        //                     .build(ctx.get_char_size());
        
        // let info_box = TextBoxBuilder::new()
        //                     .max_width(20)
        //                     .max_height(40)
        //                     .text(String::default())
        //                     .is_focused(false)
        //                     .build();

        // self.world.create_entity()
        //     .with(panel)
        //     .with(PlayerCard::new())
        //     .with(info_box)
        //     .build();

        let test_panel = PanelBuilder::new()
                            .width_percentage(80)
                            .height_exact(8)
                            .with_horiz_align(HorizontalAlignment::LEFT)
                            .with_vert_align(VerticalAlignment::TOP)
                            .is_decorated(true)
                            .build(ctx.get_char_size());

        let poem = r"
           A \n
           AB \n
           ABC \n
           ABCD \n
           ABCDE \n
           ABCDEF \n
           ABCDEFG \n
        ";


        let test_text = TextBoxBuilder::new()
                            .max_width(50)
                            .max_height(6)
                            .text(String::from(poem))
                            .is_animated(true)
                            .is_close_on_end(true)
                            .is_focused(true)
                            .build();

        self.world.create_entity()
            .with(test_panel)
            .with(test_text)
            .build();

        info!("Initialized state");
    }

    fn on_enter (&mut self) {}

    fn update (&mut self, ctx : &mut BTerm ) -> StateAction {
        {
        let mut current_input = self.world.write_resource::<PortableContext>();
        *current_input = make_portable_ctx(ctx);
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