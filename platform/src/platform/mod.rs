#[macro_use]
#[cfg(any(target_os = "linux", target_os="macos", target_os="windows"))]
pub mod cx_desktop;

#[macro_use]
pub mod cx_shared;


#[cfg(target_os = "macos")]
pub mod apple;

#[cfg(target_os = "macos")]
pub use crate::platform::apple::*;

#[cfg(target_arch = "wasm32")]
pub mod web_browser;

#[cfg(target_arch = "wasm32")]
pub use crate::platform::web_browser::*;

