#[macro_use]
extern crate log;
extern crate bracket_lib;
extern crate serde;
extern crate simplelog;
extern crate specs;
extern crate toml;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate specs_derive;
extern crate glutin;
extern crate object_pool;

pub mod components;
pub mod config;
pub mod item;
pub mod level_generation;
pub mod raw;
pub mod state;
pub mod systems;
pub mod test_state;

use bracket_lib::prelude::BTerm;
use bracket_lib::prelude::BTermBuilder;
use bracket_lib::prelude::*;
use raw::RAW;
use simplelog::*;
use state::*;
use std::fs::File;
use test_state::TestState;

fn main() {
    let settings_context = config::load_config_file();

    let filter: LevelFilter = if settings_context.settings.development.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    CombinedLogger::init(vec![
        TermLogger::new(filter, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            filter,
            Config::default(),
            File::create("polymorph.log").unwrap(),
        ),
    ])
    .unwrap();

    if settings_context.log_warning {
        warn!("{}", settings_context.message);
    } else if settings_context.log_error {
        error!("{}", settings_context.message);
    } else {
        info!("{}", settings_context.message);
    }

    let mut context: BTerm = BTermBuilder::new()
        .with_dimensions(80, 40)
        .with_tile_dimensions(8, 12)
        .with_title("Polymorph")
        .with_resource_path("assets")
        .with_fullscreen(settings_context.settings.graphical.fullscreen)
        .with_font("terminal.png", 8, 12)
        .with_vsync(settings_context.settings.graphical.vsync)
        .with_simple_console(80, 40, "terminal.png")
        .with_simple_console_no_bg(80, 40, "terminal.png")
        .build()
        .unwrap();

    let console_dimensions = config::auto_detect_resolution();
    context.set_char_size(console_dimensions.0, console_dimensions.1);
    context.set_active_console(1);
    context.set_char_size(console_dimensions.0, console_dimensions.1);
    context.set_active_console(0);

    context.post_scanlines = settings_context
        .settings
        .graphical
        .post_processing
        .scan_lines;
    context.post_screenburn = settings_context
        .settings
        .graphical
        .post_processing
        .screen_burn;

    RAW.lock().unwrap().load_raws();
    if !settings_context.settings.graphical.fullscreen {
        BACKEND
            .lock()
            .context_wrapper
            .as_ref()
            .unwrap()
            .wc
            .window()
            .set_maximized(true);
    }

    debug!("Creating Manager");
    let mut gs: Manager = Manager::new();
    gs.push(
        Box::new(TestState::new(&mut context)),
        Option::from(format!("Initial state.")),
        &mut context,
    );
    debug!("Starting main loop");
    let main_loop_result = main_loop(context, gs);
    match main_loop_result {
        Ok(_v) => {}
        Err(e) => {
            error!("Error initializing main loop : {}", e);
        }
    }
}
