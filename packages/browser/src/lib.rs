use std::{rc::Rc, cell::RefCell};

use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::CanvasRenderingContext2d;

const CELL_SIZE: f64 = 8.0;
const MAP_PX: u32 = 1024;

#[derive(Debug,Clone)]
pub struct CanvasView {
    canvas_context: CanvasRenderingContext2d,
}

impl CanvasView {
    pub fn new(canvas_context: CanvasRenderingContext2d) -> Self {
        Self { canvas_context }
    }
    pub fn draw(&self, state: &game::GameState) {
        let context = &self.canvas_context;
        context.clear_rect(0.0, 0.0, MAP_PX.into(), MAP_PX.into());
        context.begin_path();
        for (address, _) in state.cells.iter() {
            context.rect(
                address.x as f64 * CELL_SIZE,
                address.y as f64 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            );
        }
        context.fill();
    }
    pub fn init_input_receiver(&self) -> Rc<RefCell<Vec<game::Input>>> {
        let canvas = self.canvas_context.canvas().unwrap();
        let events = Rc::new(RefCell::new(Vec::new()));
        let events_for_closure = events.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let x = event.client_x() as f64 ;
            let y = event.client_y() as f64 ;
            let address = game::Address {
                x: (x / CELL_SIZE) as isize,
                y: (y / CELL_SIZE) as isize,
            };
            events_for_closure.borrow_mut().push(game::Input::Click { address });
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
        events
    } 
}