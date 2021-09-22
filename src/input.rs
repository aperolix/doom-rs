use std::{cell::RefCell, rc::Rc};

use glutin::event::VirtualKeyCode;

pub trait InputListener {
    fn on_input_change(&mut self, key: VirtualKeyCode, pressed: bool);
    fn on_mouse_move(&mut self, delta: (f64, f64));
}

pub struct Input {
    pub pressed: Vec<VirtualKeyCode>,
    pub listeners: Vec<Rc<RefCell<dyn InputListener>>>,
}

impl Input {
    pub fn new() -> Self {
        Input {
            pressed: Vec::new(),
            listeners: Vec::new(),
        }
    }
    pub fn register_input_event(&mut self, key: VirtualKeyCode, pressed: bool) {
        let mut index = 0;
        let was_pressed = self.pressed.iter().any(|&i| {
            index += 1;
            i == key
        });
        if pressed != was_pressed {
            if pressed {
                self.pressed.push(key);
            } else {
                self.pressed.remove(index - 1);
            }

            // Tell listeners
            self.listeners.iter_mut().for_each(|listener| {
                listener
                    .try_borrow_mut()
                    .unwrap()
                    .on_input_change(key, pressed)
            });
        }
    }
    pub fn register_mouse_move(&mut self, delta: (f64, f64)) {
        self.listeners
            .iter_mut()
            .for_each(|listener| listener.try_borrow_mut().unwrap().on_mouse_move(delta));
    }
}
