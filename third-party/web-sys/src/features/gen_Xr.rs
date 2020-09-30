#![allow(unused_imports)]
use super::*;
use wasm_bindgen::prelude::*;
#[cfg(web_sys_unstable_apis)]
#[wasm_bindgen]
extern "C" {
    # [wasm_bindgen (extends = EventTarget , extends = :: js_sys :: Object , js_name = XR , typescript_type = "XR")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[doc = "The `Xr` class."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/XR)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Xr`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub type Xr;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , getter , js_class = "XR" , js_name = ondevicechange)]
    #[doc = "Getter for the `ondevicechange` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/XR/ondevicechange)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Xr`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn ondevicechange(this: &Xr) -> Option<::js_sys::Function>;
    #[cfg(web_sys_unstable_apis)]
    # [wasm_bindgen (structural , method , setter , js_class = "XR" , js_name = ondevicechange)]
    #[doc = "Setter for the `ondevicechange` field of this object."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/XR/ondevicechange)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Xr`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn set_ondevicechange(this: &Xr, value: Option<&::js_sys::Function>);
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "XrSessionMode")]
    # [wasm_bindgen (method , structural , js_class = "XR" , js_name = isSessionSupported)]
    #[doc = "The `isSessionSupported()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/XR/isSessionSupported)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Xr`, `XrSessionMode`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn is_session_supported(this: &Xr, mode: XrSessionMode) -> ::js_sys::Promise;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(feature = "XrSessionMode")]
    # [wasm_bindgen (method , structural , js_class = "XR" , js_name = requestSession)]
    #[doc = "The `requestSession()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/XR/requestSession)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Xr`, `XrSessionMode`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn request_session(this: &Xr, mode: XrSessionMode) -> ::js_sys::Promise;
    #[cfg(web_sys_unstable_apis)]
    #[cfg(all(feature = "XrSessionInit", feature = "XrSessionMode",))]
    # [wasm_bindgen (method , structural , js_class = "XR" , js_name = requestSession)]
    #[doc = "The `requestSession()` method."]
    #[doc = ""]
    #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/XR/requestSession)"]
    #[doc = ""]
    #[doc = "*This API requires the following crate features to be activated: `Xr`, `XrSessionInit`, `XrSessionMode`*"]
    #[doc = ""]
    #[doc = "*This API is unstable and requires `--cfg=web_sys_unstable_apis` to be activated, as"]
    #[doc = "[described in the `wasm-bindgen` guide](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
    pub fn request_session_with_options(
        this: &Xr,
        mode: XrSessionMode,
        options: &XrSessionInit,
    ) -> ::js_sys::Promise;
}
