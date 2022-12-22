//! # 🔞
//! `r18` is a crate for internationalized Rust projects.
//! 
//! ## Usage
//! 
//! Add `r18` to your project's dependencies.
//! 
//! ```toml
//! [dependencies]
//! r18 = "0.1"
//! ```
//! Create a `JSON` translation file whose name format is `BCP 47` language tag 
//! in a directory and write it as follows:
//!
//! ```json
//! // ./tr/zh-CN.json
//! {
//!     "Hello, {}": "你好，{}"
//! }
//! ```
//!
//! Then add [`init!`] to the global area of your code with the translation 
//! file directory path (is `./tr` in this example) relative to your project root.
//!
//! ```ignore
//! r18::init!("tr");
//! ```
//!
//! After initialization, use [`auto_detect!`] to detect locale and 
//! load translation model (optional, you can use [`set_locale!`] to set locale manually),
//!  then use [`tr!`] to translate your text which has been translated.
//!
//! ```ignore
//! r18::init!("tr");
//!
//! fn main() {
//!     r18::auto_detect!();
//!
//!     let name = "ho-229";
//!     println!("{}", r18::tr!("Hello, {}", name));
//!
//!     // reset locale to disable translation
//!     r18::set_locale!("");
//!     assert_eq!("Hello, ho-229", r18::tr!("Hello, {}", name));
//! }
//! ```

use std::{collections::HashMap, sync::Mutex};

pub use dynfmt::{Format, SimpleCurlyFormat};
pub use once_cell::sync::Lazy;
pub use sys_locale::get_locale;

pub use proc_macros::init;

pub struct Locale {
    pub name: &'static str,
    pub translate: HashMap<&'static str, &'static str>,
}

pub static CURRENT_LOCALE: Lazy<Mutex<Option<&'static Lazy<Locale>>>> =
    Lazy::new(|| Mutex::new(None));

/// Translate content with the locale setting and given prefix.
/// 
/// We recommend using [`tr!`] instead of [`translate`] for translate your content.
pub fn translate<'a>(prefix: impl AsRef<str>, content: &'a str) -> &'a str {
    let locale = CURRENT_LOCALE.lock().unwrap();
    let locale = match *locale {
        Some(l) => l,
        None => return content,
    };

    match locale
        .translate
        .get(format!("{}.{}", prefix.as_ref(), content).as_str())
    {
        Some(tr) => tr,
        None => content,
    }
}

mod_use::mod_use!(macros);
