use wasm_bindgen::{JsCast, JsValue};
use web_sys::{console, window, HtmlCanvasElement, CanvasRenderingContext2d};

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

    // canvasを塗りつぶす
    context.rect(0.0, 0.0, 100.0, 100.0);
    context.rect(100.0, 100.0, 100.0, 100.0);
    context.fill();

    Ok(())
}
