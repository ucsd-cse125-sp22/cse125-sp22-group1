use std::collections::HashMap;
use std::mem::{discriminant, Discriminant};
use std::vec::Vec;

#[derive(Debug)]
pub enum Hook {
    KeyUp(u64),
    KeyDown,
}

pub struct HookManager<'a> {
    stored_funcs: HashMap<Discriminant<Hook>, Vec<&'a mut dyn FnMut(&Hook)>>,
}

impl<'a> HookManager<'a> {
    pub fn new() -> Self {
        Self {
            stored_funcs: HashMap::new(),
        }
    }

    pub fn add(&mut self, h: Hook, f: &'a mut dyn FnMut(&Hook)) {
        let d: Discriminant<Hook> = discriminant(&h);
        match self.stored_funcs.get_mut(&d) {
            Some(v) => v.push(f),
            None => {
                self.stored_funcs.insert(d, vec![f]);
            }
        }
    }

    pub fn call(&mut self, h: Hook) {
        let d: Discriminant<Hook> = discriminant(&h);
        if let Some(watchers) = self.stored_funcs.get_mut(&d) {
            for f in watchers.iter_mut() {
                f(&h);
            }
        }
    }
}
