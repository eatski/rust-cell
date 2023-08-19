use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;

const CELL_SIZE: f64 = 16.0;
const MAP_PX: u32 = 1024;

#[derive(Debug, Clone)]
pub struct CanvasView {
    canvas_context: CanvasRenderingContext2d,
}

impl CanvasView {
    pub fn new(canvas_context: CanvasRenderingContext2d) -> Self {
        Self { canvas_context }
    }
    pub fn draw(&self, state: &game::state::FinalizedGameState) {
        let context = &self.canvas_context;
        context.clear_rect(0.0, 0.0, MAP_PX.into(), MAP_PX.into());
        context.begin_path();
        for (address, unit) in state.cells.iter() {
            for path in unit.pathes.iter() {
                let address = address + path;
                context.set_fill_style(&JsValue::from_str("black"));
                context.fill_rect(
                    address.x as f64 * CELL_SIZE,
                    address.y as f64 * CELL_SIZE,
                    CELL_SIZE,
                    CELL_SIZE,
                );
            }
            
            
        }
        context.fill();

        
    }
    pub fn init_input_receiver(&self) -> Rc<RefCell<Vec<game::Input>>> {
        let canvas = self.canvas_context.canvas().unwrap();
        let events = Rc::new(RefCell::new(Vec::new()));
        let events_for_closure = events.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let x = event.offset_x() as f64;
            let y = event.offset_y() as f64;
            let address = game::state::Address {
                x: (x / CELL_SIZE) as isize,
                y: (y / CELL_SIZE) as isize,
            };
            events_for_closure
                .borrow_mut()
                .push(game::Input::Click { address });
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
        events
    }
}
