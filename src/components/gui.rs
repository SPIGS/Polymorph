use specs::{Component, VecStorage, DenseVecStorage};
use bracket_lib::prelude::{Rect, Point, RGB};

#[derive(Debug, Copy, Clone)]
pub enum HorizontalAlignment {
    RIGHT,
    LEFT,
    CENTER,
    FREE,
}

#[derive(Debug, Copy, Clone)]
pub enum VerticalAlignment {
    TOP,
    BOTTOM,
    CENTER,
    FREE,
}

pub struct PanelBuilder {
    width : i32,
    width_percent : bool,
    height : i32,
    height_percent : bool,
    horiz_align : HorizontalAlignment,
    vert_align : VerticalAlignment,
    x_offset : i32,
    y_offset : i32,
    decorated : bool,
    panel_color : RGB,
    decor_color : RGB,
    parent : Option<Rect>,
    title : Option<String>,
    title_color : RGB,
    enabled : bool,
}

impl PanelBuilder {
    pub fn new () -> Self {
        PanelBuilder {
            width : 0,
            width_percent : false,
            height : 0,
            height_percent : false,
            horiz_align : HorizontalAlignment::LEFT,
            vert_align : VerticalAlignment::TOP,
            x_offset : 0,
            y_offset : 0,
            decorated : false,
            panel_color : RGB::from_u8(0, 0, 0),
            decor_color : RGB::from_u8(255, 255, 255),
            parent : Option::None,
            title : Option::None,
            title_color : RGB::from_u8(255, 255, 255),
            enabled : true,
        }
    }

    pub fn width_exact (mut self, width : u32) -> Self {
        self.width = width as i32;
        self.width_percent = false;
        self
    }

    pub fn width_percentage (mut self, width : u32) -> Self {
        self.width = width as i32;
        self.width_percent = true;
        self
    }

    pub fn height_exact (mut self, height : u32) -> Self {
        self.height = height as i32;
        self.height_percent = false;
        self
    }

    pub fn height_percentage (mut self, height : u32) -> Self {
        self.height = height as i32;
        self.height_percent = true;
        self
    }

    pub fn with_offset (mut self, x_offset : i32, y_offset : i32) -> Self {
        self.x_offset = x_offset;
        self.y_offset = y_offset;
        self
    }

    pub fn with_horiz_align (mut self, alignment : HorizontalAlignment) -> Self {
        self.horiz_align = alignment;
        self
    }

    pub fn with_vert_align (mut self, alignment : VerticalAlignment) -> Self {
        self.vert_align = alignment;
        self
    }   

    pub fn is_decorated (mut self, decorated : bool) -> Self {
        self.decorated = decorated;
        self
    }

    pub fn color (mut self, panel_color : RGB) -> Self {
        self.panel_color = panel_color;
        self
    }

    pub fn decoration_color (mut self, decor_color : RGB) -> Self {
        self.decor_color = decor_color;
        self
    }

    pub fn parent (mut self, parent : Rect) -> Self {
        self.parent = Option::from(parent);
        self
    }

    pub fn title (mut self, title : String) -> Self {
        self.title = Option::from(title);
        self
    }

    pub fn title_color (mut self, title_color : RGB) -> Self {
        self.title_color = title_color;
        self
    }

    pub fn is_enabled (mut self, enabled : bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn build (self, screen_size : (u32, u32)) -> Panel {

        let panel_width = if self.width_percent {
            ((self.width as f32 / 100.0) * screen_size.0 as f32) as i32 - 1
        } else {
            self.width
        };
        warn!("{}", panel_width);
        let panel_height = if self.height_percent {
            ((self.height as f32 / 100.0) * screen_size.1 as f32) as i32 - 1
            
        } else {
            self.height
        };


        let x : i32;
        match self.horiz_align {
            HorizontalAlignment::RIGHT => x = screen_size.0 as i32 - (panel_width+1) + self.x_offset,
            HorizontalAlignment::LEFT => x = 0 + self.x_offset,
            HorizontalAlignment::CENTER => x = (screen_size.0 as i32 / 2) - (panel_width / 2 + 1) + self.x_offset,
            HorizontalAlignment::FREE => x = self.x_offset, 
        }

        let y : i32;
        match self.vert_align {
            VerticalAlignment::BOTTOM => y = screen_size.1 as i32 - (panel_height + 1) + self.y_offset,
            VerticalAlignment::TOP => y = 0 + self.y_offset,
            VerticalAlignment::CENTER => y = (screen_size.1 as i32 / 2) - (panel_height / 2 + 1) + self.y_offset,
            VerticalAlignment::FREE => y = self.y_offset, 
        }

        Panel {
            width : panel_width,
            height : panel_height,
            x : x,
            y : y,
            horiz_align : self.horiz_align,
            vert_align : self.vert_align,
            decorated : self.decorated,
            panel_color : self.panel_color,
            decor_color : self.decor_color,
            parent : self.parent,
            title : self.title,
            title_color : self.title_color,
            enabled : self.enabled,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Panel {
    pub width : i32,
    pub height : i32,
    pub x: i32,
    pub y: i32,
    horiz_align : HorizontalAlignment,
    vert_align : VerticalAlignment,
    pub decorated : bool,
    pub panel_color : RGB,
    pub decor_color : RGB,
    pub parent : Option<Rect>,
    pub title : Option<String>,
    pub title_color : RGB,
    pub enabled : bool,
}

impl Panel {
    pub fn vertical_center (&self) -> i32 {
        return (self.height / 2) + self.y;
    }

    pub fn horizontal_center (&self) -> i32 {
        return (self.width / 2) + self.x;
    }

    pub fn get_horiz_align (&self) -> HorizontalAlignment {
        self.horiz_align
    }

    pub fn get_vert_align (&self) -> VerticalAlignment {
        self.vert_align
    }

    pub fn set_horiz_align (&mut self, alignment : HorizontalAlignment, screen_width : u32) {

        let new_x : i32;
        //update the relevant coordinate
        match alignment {
            HorizontalAlignment::RIGHT => new_x = screen_width as i32 - (self.width+1),
            HorizontalAlignment::LEFT => new_x = 0,
            HorizontalAlignment::CENTER => new_x = (screen_width as i32 / 2) - self.horizontal_center(),
            HorizontalAlignment::FREE => new_x = self.x, 
        }

        self.horiz_align = alignment;
        self.x = new_x;
    }

    pub fn set_vert_align (&mut self, alignment : VerticalAlignment, screen_height : u32) {

        let new_y : i32;
        //update the relevant coordinate
        match alignment {
            VerticalAlignment::BOTTOM => new_y = screen_height as i32 - (self.height + 1),
            VerticalAlignment::TOP => new_y = 0,
            VerticalAlignment::CENTER => new_y = (screen_height as i32 / 2) - self.vertical_center(),
            VerticalAlignment::FREE => new_y = self.y, 
        }

        self.vert_align = alignment;
        self.y = new_y;
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PlayerCard {
    pub alignment : HorizontalAlignment,
}

impl PlayerCard {

    pub fn new () -> Self {
        PlayerCard {
            alignment : HorizontalAlignment::RIGHT
        }
    }

    pub fn cycle_alignment (&mut self, panel : &mut Panel, screen_width : u32) {
        
        // this component changes another component! THIS IS BAD!!
        match panel.get_horiz_align() {
            HorizontalAlignment::RIGHT => {
                panel.set_horiz_align(HorizontalAlignment::LEFT, screen_width);
                self.alignment = HorizontalAlignment::LEFT;
            },
            HorizontalAlignment::LEFT => {
                panel.set_horiz_align(HorizontalAlignment::FREE, screen_width);
                self.alignment = HorizontalAlignment::FREE;
                panel.enabled = false;
            },
            _ => {
                panel.set_horiz_align(HorizontalAlignment::RIGHT, screen_width);
                self.alignment = HorizontalAlignment::RIGHT;
                panel.enabled = true;
            },
        }
    }
}


pub struct TextBoxBuilder {
    max_width : usize,
    max_height : usize,
    raw_text : String,
    animated : bool,
    focused : bool,
}

impl TextBoxBuilder {
    pub fn new () -> Self {
        TextBoxBuilder {
            max_width : 0,
            max_height : 0,
            raw_text : String::default(),
            animated : false,
            focused : true,
        }
    }

    pub fn max_width (mut self, width : usize) -> Self {
        self.max_width = width;
        self
    }

    pub fn max_height (mut self, height : usize) -> Self {
        self.max_height = height;
        self
    }

    pub fn text (mut self, text : String) -> Self {
        self.raw_text = text.replace('\n', "");
        self.raw_text = self.raw_text.replace('\t', "");
        self.raw_text = self.raw_text.replace('\r', "");
        self
    }

    pub fn is_animated (mut self, animated : bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn is_focused (mut self, focused : bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn build (self) -> TextBox{
        let tokens = self.raw_text.split_whitespace();

        let mut lines : Vec<String> = Vec::new();
        let mut cur_line : String = String::default();
        for token in tokens {
            if token == r"\n" {
                lines.push(cur_line.clone());
                cur_line = String::default();
            } else {
                if token.len() + cur_line.len() <= self.max_width {
                    cur_line.push_str(token);
                    cur_line.push(' ');
                } else {
                    lines.push(cur_line.clone());
                    cur_line = String::default();
                }
            }
        }
        
        lines.push(cur_line.clone());
        let pages = lines.len() / self.max_height;
        TextBox {
            lines : lines,
            max_width : self.max_width,
            max_height : self.max_height,
            is_animated : self.animated,
            done_animating : !self.animated,
            is_waiting : false,
            accumulator : 0.0,
            rate: 50.0,
            character : 0,
            line : 0,
            page : 0,
            total_pages : pages,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct TextBox {
    pub lines : Vec<String>,
    pub max_width : usize,
    pub max_height : usize,
    pub is_animated : bool,
    pub done_animating : bool,
    pub is_waiting : bool,
    accumulator : f32,
    rate : f32,
    character : usize,
    line : usize,
    page : usize,
    total_pages : usize,
}

impl TextBox {
    pub fn update_text (&mut self, new_text : String) {
        let mut raw_text = new_text;
        raw_text = raw_text.replace('\n', "");
        raw_text = raw_text.replace('\t', "");
        raw_text = raw_text.replace('\r', "");

        let tokens = raw_text.split_whitespace();

        let mut new_lines : Vec<String> = Vec::new();
        let mut cur_line : String = String::default();
        for token in tokens {
            if token == r"\n" {
                new_lines.push(cur_line.clone());
                cur_line = String::default();
            } else {
                if token.len() + cur_line.len() <= self.max_width {
                    cur_line.push_str(token);
                    cur_line.push(' ');
                } else {
                    new_lines.push(cur_line.clone());
                    cur_line = String::default();
                }
            }
        }
        new_lines.push(cur_line.clone());

        self.lines = new_lines;
    }

    pub fn animate_step (&mut self, delta : f32) {
        self.accumulator += delta;
        if !self.is_waiting {
            if self.accumulator / self.rate >= 1.0 {
                    self.character += 1;
                    if self.character == self.lines[self.line].len() {
                        self.line += 1;
                        self.character = 0;
                        if self.line == self.max_height {
                            self.is_waiting = true;
                            warn!("waiting");
                        }
                    }
            
                self.accumulator = 0.0;
            }
        }
        let adj_line = self.page * self.max_height;
        if self.character >= self.lines[self.lines.len()-1].len() && (adj_line + self.line) >= self.lines.len() && self.page >= self.total_pages {
            self.done_animating = true;
            warn!("Animation done!!");
        }
    }

    pub fn proceed (&mut self) {
        self.is_waiting = false;
        self.page += 1;
        self.line = 0;
    }

    pub fn current_line (&self) -> usize {
        self.line
    }

    pub fn current_character (&self) -> usize {
        self.character
    }

    pub fn current_page (&self) -> usize {
        self.page
    }

    pub fn total_pages (&self) -> usize {
        self.total_pages
    }
}

