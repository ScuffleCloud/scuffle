use wasm_bindgen::prelude::*;

mod hls;
mod player;
mod tracing_wasm;

#[wasm_bindgen(main)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();

    Ok(())
}

#[cfg(test)]
mod tests;
