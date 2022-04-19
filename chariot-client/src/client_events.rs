use winit::event::*;

pub trait Watching {
    fn on_key_down(&mut self, key: VirtualKeyCode);
    fn on_key_up(&mut self, key: VirtualKeyCode);

    fn on_mouse_move(&mut self, x: f64, y: f64);
    fn on_left_mouse(&mut self, x: f64, y: f64, state: ElementState);
    fn on_right_mouse(&mut self, x: f64, y: f64, state: ElementState);
}
