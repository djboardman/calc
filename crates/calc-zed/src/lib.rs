mod configuration;
mod extension;
mod language_server_locator;

use zed_extension_api as zed;

pub use extension::CalcZed;

zed::register_extension!(CalcZed);
