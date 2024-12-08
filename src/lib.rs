mod state;

#[macro_export]
macro_rules! entrypoint {
    ($name:ident) => {
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
        #[allow(dead_code)]
        fn wasm_entrypoint() {
            $name()
        }
    };
}
