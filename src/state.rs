use bracket_lib::prelude::{BTerm, GameState, VirtualKeyCode};
use std::collections::HashMap;
use std::collections::VecDeque;

pub enum StateAction {
    NoAction,
    Push(Box<dyn State>, Option<String>),
    Pop(Option<String>),
    PopAmount(u32, Option<String>),
    Switch(Box<dyn State>, Option<String>),
    Exit,
}

pub enum WorldAction {
    NoAction,
    PassInventory(HashMap<u32, u32>),
    PlayerEquipItem(u32),
}

///read only context that can be cloned and passed between threads and sytems safely
pub struct PortableContext {
    pub fps: f32,
    pub delta: f32,
    pub key: Option<VirtualKeyCode>,
    pub mouse_pos: (i32, i32),
    pub left_click: bool,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub quitting: bool,
    pub screen_size : (u32, u32),
}

impl Default for PortableContext {
    fn default() -> Self {
        PortableContext {
            fps: 0.0,
            delta: 0.0,
            key: None,
            mouse_pos: (0, 0),
            left_click: false,
            shift: false,
            control: false,
            alt: false,
            quitting: false,
            screen_size : (0, 0),
        }
    }
}

pub trait State {
    /// Called when this state is pushed to the stack.
    fn init(&mut self, ctx : &mut BTerm);
    /// Called when this state become the top of the stack.
    fn on_enter(&mut self);
    /// Called routinely.
    fn update(&mut self, ctx: &mut BTerm ) -> StateAction;
    /// Called routinely after update.
    fn render(&mut self, ctx: &mut BTerm);
    /// Called this state is popped or switched.
    fn on_exit(&mut self);
}

pub struct Manager {
    pub states: VecDeque<Box<dyn State>>,
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            states: VecDeque::new(),
        }
    }

    /// Pushes a new state to the top of the stack.
    pub fn push(&mut self, state: Box<dyn State>, message: Option<String>, ctx : &mut BTerm) {
        info!("Pushing state");
        match message {
            Some(msg) => {
                debug!("{}", msg);
            }
            _ => {}
        }
        self.states.push_front(state);
        self.states[0].init(ctx);
    }

    /// Pops the state from the top of the stack. If no states are left on the stack,
    /// it exits the program.
    pub fn pop(&mut self, message: Option<String>) {
        info!("Popping state");
        match message {
            Some(msg) => {
                debug!("{}", msg);
            }
            _ => {}
        }
        self.states[0].on_exit();
        self.states.pop_front();
        if self.states.len() >= 1 {
            self.states[0].on_enter();
        }
    }

    /// Pops a number of states from the top of the stack. If no states are left on the stack,
    /// it exits the program.
    pub fn pop_amount(&mut self, amt: u32, message: Option<String>) {
        info!("Popping {} state(s)", amt);
        match message {
            Some(msg) => {
                debug!("{}", msg);
            }
            _ => {}
        }
        for _i in 0..amt {
            self.pop(Option::None);
        }
    }

    /// Pops and replaces the state at the top of the stack with a new state.
    pub fn switch(&mut self, state: Box<dyn State>, message: Option<String>, ctx : &mut BTerm) {
        info!("Switching states");
        match message {
            Some(msg) => {
                debug!("{}", msg);
            }
            _ => {}
        }
        self.pop(Option::None);
        self.push(state, Option::None, ctx);
        info!("Switching state")
    }

    /// Pops all states of the stack and exits the program.
    pub fn exit(&mut self) {
        let length = self.states.len();

        for _i in 0..length {
            self.states.pop_front();
        }
        info!("Exiting...")
    }
}

impl GameState for Manager {
    fn tick(&mut self, ctx: &mut BTerm) {
        let action = self.states[0].update(ctx);
        self.states[0].render(ctx);

        match action {
            StateAction::NoAction => {}
            StateAction::Pop(msg) => self.pop(msg),
            StateAction::PopAmount(number, msg) => self.pop_amount(number, msg),
            StateAction::Push(new_state, msg) => self.push(new_state, msg, ctx),
            StateAction::Switch(new_state, msg) => self.switch(new_state, msg, ctx),
            StateAction::Exit => self.exit(),
        }

        if self.states.len() == 0 {
            ctx.quit();
            info!("Terminated.");
        }
    }
}

pub fn make_portable_ctx (ctx: &mut BTerm) -> PortableContext {
    PortableContext {
        fps: ctx.fps,
        delta: ctx.frame_time_ms,
        key: ctx.key,
        mouse_pos: ctx.mouse_pos,
        left_click: ctx.left_click,
        shift: ctx.shift,
        control: ctx.control,
        alt: ctx.alt,
        quitting: ctx.quitting,
        screen_size : ctx.get_char_size(),
    }
}

pub mod time {
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Gets the current time in milliseconds from the epoch.
    pub fn get_current_time_millis() -> u128 {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Contact Einstein.");
        return since_epoch.as_millis();
    }

    /// Gets the current time in nanoseconds from the epoch.
    pub fn get_current_time_nano() -> u128 {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Contact Einstein");
        return since_epoch.as_nanos();
    }
}
