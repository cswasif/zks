use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn test_string() -> JsValue {
    JsValue::from_str("0")
}