use std::path::{Path, PathBuf};

use glob::glob;
use quote::{format_ident, quote};

struct PathStr(String);

impl syn::parse::Parse for PathStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::LitStr>().map(|v| Self(v.value()))
    }
}

/// Generate translation models and functions `set_locale` and `locale` to setup
/// `r18` environment with given translation directory.
///
/// ## Example
///
/// ```ignore
/// r18::init!("tr");
/// ```
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

    let mut languages = vec![];
    let locales = glob(&format!("{}/**/*.json", path.display()))
        .expect("Failed to read glob pattern")
        .filter_map(|p| p.ok())
        .map(|p| {
            let language = p
                .file_stem()
                .and_then(|p| p.to_str())
                .and_then(|p| p.split('.').last())
                .expect("Cannot find language code")
                .to_string();

            let ret = generate_one_locale(&language, p);
            languages.push(language);
            ret
        })
        .collect::<proc_macro2::TokenStream>();

    let locale_helpers = generate_helpers(&languages);

    quote! {
        #[doc(hidden)]
        pub(crate) mod __r18_gen {
            #locales
            #locale_helpers
        }
    }
    .into()
}

fn generate_one_locale(language: &str, path: impl AsRef<Path>) -> proc_macro2::TokenStream {
    let code = format_ident!("{}", language.to_uppercase().replace('-', "_"));
    let translation = r18_trans_support::import(path)
        .into_iter()
        .map(|(k, v)| quote!( #k => #v ));
    // let translation = quote!( );

    quote! {
        #[doc(hidden)]
        const #code: r18::Locale = r18::Locale {
            name: #language,
            translate: {
                use r18::phf;
                phf::phf_map! {
                    #( #translation ),*
                }
            }
        };
    }
}

fn generate_helpers(languages: &[String]) -> proc_macro2::TokenStream {
    let lang_idents = languages
        .iter()
        .map(|l| format_ident!("{}", l.to_uppercase().replace('-', "_")))
        .collect::<Vec<_>>();

    quote! {
        #[doc(hidden)]
        pub(crate) fn set_locale(locale: impl AsRef<str>) {
            *r18::CURRENT_LOCALE.lock().unwrap() = match locale.as_ref() {
                #(#languages => Some(& #lang_idents),)*
                _ => None,
            };
        }
    }
}
