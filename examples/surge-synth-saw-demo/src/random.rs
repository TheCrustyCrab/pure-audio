#[cfg(target_arch = "wasm32")]
pub(crate) fn get_random(max_exclusive: usize) -> usize {
    // getrandom uses crypto.getRandomValues, which is not available in the AudioWorkletGlobalScope
    (pure_audio_wasm::js_sys::Math::random() * max_exclusive as f64).floor() as usize
}


#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn get_random(max_exclusive: usize) -> usize {
    (getrandom::u32().unwrap() % max_exclusive as u32) as usize
}