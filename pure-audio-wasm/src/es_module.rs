use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub(crate) type ImportMeta;

    #[wasm_bindgen(method, getter)]
    pub(crate) fn url(this: &ImportMeta) -> JsString;

    #[wasm_bindgen(thread_local, js_namespace = import, js_name = meta)]
    pub(crate) static IMPORT_META: ImportMeta;
}