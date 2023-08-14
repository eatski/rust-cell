use std::{cell::RefCell, rc::Rc};

use browser::CanvasView;
use game::{update, GameState};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{console, window, CanvasRenderingContext2d, HtmlCanvasElement};

fn get_window() -> web_sys::Window {
    web_sys::window().expect("should have a window in this context")
}

const MAP_PX: u32 = 1024;
fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console::log_1(&"Hello world!".into());
    // body配下にcanvasを追加
    let window = window().ok_or_else(|| JsValue::from_str("No global `window` exists"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("The window does not have a document"))?;
    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("The document does not have a body"))?;
    let canvas = document.create_element("canvas")?;
    body.append_child(&canvas)?;

    // canvasを初期化
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;
    canvas.set_width(MAP_PX);
    canvas.set_height(MAP_PX);
    canvas.set_attribute("style", "border: 1px solid black;cursor: pointer;")?;
    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("The canvas does not have a 2d context"))?;
    let context: CanvasRenderingContext2d = context.dyn_into()?;

    let mut state = GameState::default();

    let drawer = CanvasView::new(context);

    let events = drawer.init_input_receiver();

    // pointsをcalc_next_pointsしながら繰り返し描画する
    set_interval_with_request_animation_frame(
        move |events| {
            drawer.draw(&state);
            update(&mut state, &events);
        },
        events,
    );

    Ok(())
}

const FRAME_SIZE: f64 = 1000.0 / 30.0;

/**
 * 任意のFnMutをrequest_animation_frame関数を使って繰り返し呼び出す。
 * 再帰は使わず、Rcで参照を保持する。
 */
fn set_interval_with_request_animation_frame<Input: 'static>(
    mut frame: impl FnMut(Vec<Input>) + 'static,
    events: Rc<RefCell<Vec<Input>>>,
) {
    let mut last = get_window().performance().unwrap().now();
    let mut acc = 0.0;

    type LoopClosure = Closure<dyn FnMut(f64)>;
    fn request_animation_frame(closure: &LoopClosure) {
        get_window()
            .request_animation_frame(closure.as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }
    let frame_rc: Rc<RefCell<Option<LoopClosure>>> = Rc::new(RefCell::new(None));
    let frame_rc_clone: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = frame_rc.clone();
    let closure = Closure::wrap(Box::new(move |now| {
        let dt = now - last;
        acc += dt;

        while acc >= FRAME_SIZE {
            let taked = events.borrow_mut().drain(..).collect::<Vec<_>>();
            frame(taked);
            acc -= FRAME_SIZE;
        }
        last = now;
        request_animation_frame(frame_rc.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>);
    *frame_rc_clone.borrow_mut() = Some(closure);
    request_animation_frame(frame_rc_clone.borrow().as_ref().unwrap());
}
