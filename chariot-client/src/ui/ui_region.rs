pub struct UIRegion {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    is_hovering: bool,
    is_active: bool,
    on_enter_listeners: Vec<Box<dyn FnMut() -> ()>>,
    on_exit_listeners: Vec<Box<dyn FnMut() -> ()>>,
    on_click_listeners: Vec<Box<dyn FnMut() -> ()>>,
    on_release_listeners: Vec<Box<dyn FnMut() -> ()>>,
}

impl UIRegion {
    // since winit uses pixels this region must be defined in pixels as well
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> UIRegion {
        UIRegion {
            x0: x,
            x1: x + width,
            y0: y,
            y1: y + height,
            is_hovering: false,
            is_active: false,
            on_enter_listeners: Vec::new(),
            on_exit_listeners: Vec::new(),
            on_click_listeners: Vec::new(),
            on_release_listeners: Vec::new(),
        }
    }

    #[inline]
    fn execute_on_enter(&mut self) {
        self.on_enter_listeners
            .iter_mut()
            .for_each(|boxed| (**boxed)());
    }

    #[inline]
    fn execute_on_exit(&mut self) {
        self.on_exit_listeners
            .iter_mut()
            .for_each(|boxed| (**boxed)());
    }

    #[inline]
    fn execute_on_click(&mut self) {
        self.on_click_listeners
            .iter_mut()
            .for_each(|boxed| (**boxed)());
    }

    #[inline]
    fn execute_on_release(&mut self) {
        self.on_release_listeners
            .iter_mut()
            .for_each(|boxed| (**boxed)());
    }

    #[inline]
    fn is_inside(&self, x: f64, y: f64) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }

    pub fn set_hovering(&mut self, x: f64, y: f64) {
        let is_hovering = self.is_inside(x, y);

        if is_hovering == true && self.is_hovering == false {
            self.execute_on_enter();
        } else if is_hovering == false && self.is_hovering == true {
            self.execute_on_exit();
        }
        self.is_hovering = is_hovering;
    }

    pub fn set_active(&mut self, x: f64, y: f64) {
        if !self.is_inside(x, y) {
            return;
        }

        self.is_active = true;
        self.execute_on_click();
    }

    pub fn set_inactive(&mut self) {
        if !self.is_active {
            return;
        }

        self.is_active = false;
        self.execute_on_release();
    }

    pub fn on_enter<F: 'static + FnMut() -> ()>(&mut self, closure: F) {
        self.on_enter_listeners.push(Box::new(closure));
    }

    pub fn on_exit<F: 'static + FnMut() -> ()>(&mut self, closure: F) {
        self.on_exit_listeners.push(Box::new(closure));
    }

    pub fn on_click<F: 'static + FnMut() -> ()>(&mut self, closure: F) {
        self.on_click_listeners.push(Box::new(closure));
    }

    pub fn on_release<F: 'static + FnMut() -> ()>(&mut self, closure: F) {
        self.on_release_listeners.push(Box::new(closure));
    }
}
