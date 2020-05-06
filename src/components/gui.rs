use specs::{Component, VecStorage, DenseVecStorage};
use bracket_lib::prelude::{Rect, Point};

#[derive(Debug, Copy, Clone)]
pub enum Justification {
    RIGHT,
    LEFT,
    CENTER,
    FREE,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Panel {
    pub bounds : Rect,
    pub decorations : bool,
    pub justification : Justification,
    pub parent : Option<Rect>,
}

impl Panel {
    pub fn new (bounds : Rect, decorations : bool, justification : Justification, parent : Option<Rect>) -> Self {
        Panel {
            bounds : bounds,
            decorations : decorations,
            justification : justification,
            parent : parent,
        }
    }

    pub fn vertical_center (&self) -> i32 {
        return self.bounds.center().y;
    }

    pub fn horizontal_center (&self) -> i32 {
        return self.bounds.center().x;
    }

    pub fn point_in_panel (&self, position : (i32,i32)) -> bool {
        return self.bounds.point_in_rect(Point::from_tuple(position));
    }

    pub fn width (&self) -> i32{
        return self.bounds.width();
    }

    pub fn height (&self) -> i32 {
        return self.bounds.height();
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PlayerCard {
    pub justification : Justification,
}

impl PlayerCard {
    pub fn new () -> Self {
        PlayerCard {
            justification : Justification::RIGHT,
        }
    }

    pub fn cycle_justification (&mut self) {
        let new_justification : Justification;
        match self.justification {
            Justification::RIGHT => new_justification = Justification::LEFT,
            Justification::LEFT => new_justification = Justification::FREE,
            _ => new_justification = Justification::RIGHT,
        }
        self.justification = new_justification;
    }
}

