use std::path::{Path, PathBuf};

use glob::glob;
use quote::{format_ident, quote};

struct PathStr(String);

impl syn::parse::Parse for PathStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::LitStr>().map(|v| Self(v.value()))
    }
}

#[proc_macro]
pub fn init(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path = match syn::parse::<PathStr>(input) {
        Ok(dir) => {
            let p = std::env::var("CARGO_MANIFEST_DIR")
                .map(|p| PathBuf::from(p).join(dir.0))
                .expect("CARGO_MANIFEST_DIR doesn't exist");

            match p.is_dir() {
                true => p,
                false => panic!("{} is not a directory.", p.display()),
            }
        }
        Err(e) => return e.to_compile_error().into(),
    };

    let mut token = proc_macro2::TokenStream::new();
    let mut languages = Vec::new();

    glob(&format!("{}/**/*.json", path.display()))
        .expect("Failed to read glob pattern")
        .filter_map(|p| p.ok())
        .for_each(|p| {
            let language = p
                .file_stem()
                .and_then(|p| p.to_str())
                .and_then(|p| p.split('.').last())
                .expect("Cannot found language code")
                .to_string();

            token.extend(generate_language(&language, p).into_iter());
            languages.push(language);
        });

    token.extend(generate_locale(languages).into_iter());

    token.into()
}

fn generate_language(language: &str, path: impl AsRef<Path>) -> proc_macro2::TokenStream {
    let code = format_ident!("{}", language.to_uppercase());
    let translation = trans_support::import(path)
        .into_iter()
        .map(|(k, v)| quote!( (#k, #v) ))
        .collect::<Vec<_>>();

    quote! {
        static #code: r18::Lazy<r18::Locale> = r18::Lazy::new(|| r18::Locale {
            name: #language,
            translate: [ #(#translation),* ]
                .into_iter()
                .collect(),
        });
    }
}

fn generate_locale(languages: Vec<String>) -> proc_macro2::TokenStream {
    let lang_idents = languages
        .iter()
        .map(|l| format_ident!("{}", l.to_uppercase()))
        .collect::<Vec<_>>();

    quote! {
        fn set_locale(locale: impl AsRef<str>) {
            *r18::CURRENT_LOCALE.lock().unwrap() = match locale.as_ref() {
                #(#languages => Some(& #lang_idents),)*
                _ => None,
            };
        }

        fn locale() -> Option<&'static str> {
            r18::CURRENT_LOCALE.lock().unwrap().as_ref().map(|l| l.name)
        }
    }
}
