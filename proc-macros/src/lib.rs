use std::path::{Path, PathBuf};

use glob::glob;
use quote::{format_ident, quote};

use oxilangtag::LanguageTag;

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
        .filter_map(|p| {
            let path = p.ok()?;

            let language = path
                .file_stem()
                .and_then(|p| p.to_str())
                .and_then(|p| p.split('.').last())
                .and_then(|p| LanguageTag::parse_and_normalize(&p).ok())?;

            let ret = generate_one_locale(&language, path);
            languages.push(language);
            Some(ret)
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

fn generate_helpers(languages: &[LanguageTag<String>]) -> proc_macro2::TokenStream {
    let lang_match = languages
        .iter()
        .fold(proc_macro2::TokenStream::new(), |mut stream, lang| {
            let ident = format_ident!("{}", (&lang).to_uppercase().replace("-", "_"));
            let (primary, region) = (lang.primary_language(), lang.region());

            if let Some(region) = region {
                stream.extend(quote! { (#primary, Some(#region)) => Some(& #ident) , }.into_iter());
            }

            stream.extend(quote! { (#primary, None) => Some(& #ident) , }.into_iter());

            stream
        });

    quote! {
        #[doc(hidden)]
        pub(crate) fn set_locale(locale: impl AsRef<str>) {
            *r18::CURRENT_LOCALE
                .lock()
                .unwrap() = match r18::LanguageTag::parse_and_normalize(locale.as_ref()) {
                Ok(lang) => {
                    match (lang.primary_language(), lang.region()) {
                        #lang_match
                        _ => None,
                    }
                }
                Err(_) => None,
            };
        }
    }
}
