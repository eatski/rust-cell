use std::{rc::Rc, cell::RefCell};

use rand::Rng;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{console, window, HtmlCanvasElement, CanvasRenderingContext2d};

fn get_window() -> web_sys::Window {
    web_sys::window().expect("should have a window in this context")
}

fn main() -> Result<(), JsValue> {
    console::log_1(&"Hello world!".into());
    // body配下にcanvasを追加
    let window = window().ok_or_else(|| JsValue::from_str("No global `window` exists"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("The window does not have a document"))?;
    let body = document.body().ok_or_else(|| JsValue::from_str("The document does not have a body"))?;
    let canvas = document.create_element("canvas")?;
    body.append_child(&canvas)?;

    // canvasを初期化
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;
    canvas.set_width(512);
    canvas.set_height(512);
    let context = canvas.get_context("2d")?.ok_or_else(|| JsValue::from_str("The canvas does not have a 2d context"))?;
    let context: CanvasRenderingContext2d = context.dyn_into()?;

    let mut points = vec![
        Point { x: 0, y: 0 },
        Point { x: 16, y: 16 },
        Point { x: 31, y: 31 },
    ];

    let events = init_input_receiver(&canvas);

    // pointsをcalc_next_pointsしながら繰り返し描画する
    set_interval_with_request_animation_frame(move |events| {
        draw_points(&context, &points);
        points = update(&points,events);
    },events);

    Ok(())
}

#[derive(Debug)]
struct Point {
    x: isize,
    y: isize,
}

const CELL_SIZE: f64 = 16.0;

/**
 * 今までの点を消去
 * 16pxを1単位として、点を描画する
 * context.rectを使用し、16pxの正方形をpointの数だけ描画する
 */
fn draw_points(context: &CanvasRenderingContext2d, points: &[Point]) {
    context.clear_rect(0.0, 0.0, 512.0, 512.0);
    context.begin_path();
    for point in points {
        context.rect(point.x as f64 * CELL_SIZE, point.y as f64 * CELL_SIZE, CELL_SIZE, CELL_SIZE);
    }
    context.fill();
}

const FRAME_SIZE : f64 = 1000.0 / 30.0;

/**
 * 任意のFnMutをrequest_animation_frame関数を使って繰り返し呼び出す。
 * 再帰は使わず、Rcで参照を保持する。
 */
fn set_interval_with_request_animation_frame(
    mut frame: impl FnMut(Vec<Point>) + 'static,
    events: Rc<RefCell<Vec<Point>>>,
) {
 

    let mut last = get_window().performance().unwrap().now();
    let mut acc = 0.0;

    type LoopClosure = Closure<dyn FnMut(f64)>;
    fn request_animation_frame(closure: &LoopClosure) {
        get_window().request_animation_frame(closure.as_ref().unchecked_ref()).expect("should register `requestAnimationFrame` OK");
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

/**
 * 次の点を計算する
 * 点はランダムに上下左右に1単位動く
 */
fn update(points: &[Point],events: Vec<Point>) -> Vec<Point> {
    let mut next_points = Vec::new();
    for point in points {
        let mut rng = rand::thread_rng();
        let direction = rng.gen_range(0..4);
        let next_point = match direction {
            0 => Point { x: point.x, y: point.y - 1 },
            1 => Point { x: point.x, y: point.y + 1 },
            2 => Point { x: point.x - 1, y: point.y },
            3 => Point { x: point.x + 1, y: point.y },
            _ => panic!("direction is invalid"),
        };
        next_points.push(next_point);
    }
    next_points.extend(events);
    next_points
}

/**
 * Canvasへのクリックイベントを受け取る
 * イベントに応じて更新されるPointの配列を返す
 */
fn init_input_receiver(canvas: &HtmlCanvasElement) -> Rc<RefCell<Vec<Point>>> {
    let points = Rc::new(RefCell::new(Vec::new()));
    let points_clone = points.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let x = event.offset_x();
        let y = event.offset_y();
        let point = Point {
            x: (x as f64 / CELL_SIZE) as isize,
            y: (y as f64 / CELL_SIZE) as isize,
        };
        points_clone.borrow_mut().push(point);
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
    points
}