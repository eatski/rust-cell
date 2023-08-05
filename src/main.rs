use wasm_bindgen::JsCast;
use web_sys::console;

fn main() {
    console::log_1(&"Hello world!".into());
    // body配下にcanvasを追加
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let canvas = document.create_element("canvas").unwrap();
    body.append_child(&canvas).unwrap();
    // canvasを初期化
    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
    // canvasを塗りつぶす
    context.begin_path();
    context.rect(0.0, 0.0, 100.0, 100.0);
    context.fill();
}
