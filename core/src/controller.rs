#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum Button {
    Start,
    A,
    B,
    DUp,
    DDown,
    DLeft,
    DRight,
    CUp,
    CDown,
    CLeft,
    CRight,
    LeftTrigger,
    RightTrigger,
    Z,
}

#[derive(Default, Clone, Copy)]
pub struct Controller {
    buttons: [bool; 14],
    x: f32,
    y: f32,
}

impl Controller {
    pub fn press(&mut self, button: Button) {
        self.buttons[button as usize] = true;
    }

    pub fn release(&mut self, button: Button) {
        self.buttons[button as usize] = false;
    }

    pub fn pressed(&self, button: Button) -> bool {
        self.buttons[button as usize]
    }

    pub fn set_axis(&mut self, x: bool, value: f32) {
        if x {
            self.x = value;
        } else {
            self.y = value;
        }
    }
}
