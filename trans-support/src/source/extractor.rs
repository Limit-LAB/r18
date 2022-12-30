use std::{collections::HashSet, fs::File, io::Read, path::Path};

use syn::__private::{quote::__private::TokenTree, ToTokens, TokenStream2};

pub fn extract(
    path: impl AsRef<Path>,
    contents: &mut HashSet<String>,
    locale: &mut String,
) -> crate::Result<()> {
    let mut file = File::open(path.as_ref())?;

    let mut source = String::with_capacity(4096);
    file.read_to_string(&mut source)?;

    let stream = syn::parse_file(&source)?.into_token_stream();

    extract_inner(stream, contents, locale)
        .map_err(|e| format!("{}:{}", path.as_ref().display(), e).into())
}

fn extract_inner(
    stream: TokenStream2,
    contents: &mut HashSet<String>,
    locale: &mut String,
) -> crate::Result<()> {
    let mut stream = stream.into_iter().peekable();

    while let Some(token) = stream.next() {
        match token {
            TokenTree::Group(g) => extract_inner(g.stream(), contents, locale)?,
            TokenTree::Ident(id) => {
                // is macro
                match stream.peek() {
                    Some(TokenTree::Punct(p)) if p.to_string() == "!" => stream.next(),
                    _ => continue,
                };

                match (id.to_string().as_str(), stream.peek()) {
                    ("init", Some(TokenTree::Group(g))) => {
                        extract_init(g.stream(), locale)?;
                        stream.next();
                    }
                    ("tr", Some(TokenTree::Group(g))) => {
                        extract_tr(g.stream(), contents)?;
                        stream.next();
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }

    Ok(())
}

fn extract_init(stream: TokenStream2, locale: &mut String) -> crate::Result<()> {
    match stream.into_iter().next() {
        Some(TokenTree::Literal(literal)) => {
            let pos = literal.span().start();

            if !locale.is_empty() {
                return Err(format!("{}:{} Multiple init! detected", pos.line, pos.column).into());
            }

            *locale = syn::parse2::<syn::LitStr>(literal.into_token_stream())
                .map_err(|e| {
                    format!(
                        "{}:{} failed to parse translation directory path: {}",
                        pos.line, pos.column, e
                    )
                })?
                .value();
        }
        Some(token) => {
            let pos = token.span().start();
            return Err(format!("{}:{} Invalid init! arguments", pos.line, pos.column).into());
        }
        _ => return Err(" Missing translation directory".into()),
    };

    Ok(())
}

fn extract_tr(stream: TokenStream2, contents: &mut HashSet<String>) -> crate::Result<()> {
    let mut stream = stream.into_iter().peekable();

    let prefix = match stream.peek() {
        Some(TokenTree::Group(g)) => match g.stream().into_iter().next() {
            Some(TokenTree::Literal(literal)) => {
                let pos = literal.span().start();
                let ret = syn::parse2::<syn::LitStr>(literal.into_token_stream())
                    .map_err(|e| {
                        format!("{}:{} Invalid prefix syntax: {}", pos.line, pos.column, e)
                    })?
                    .value();

                if !ret.starts_with('.') {
                    return Err(format!(
                        "{}:{} Invalid prefix syntax: prefix must start with '.'",
                        pos.line, pos.column
                    )
                    .into());
                }

                stream.next();
                ret
            }
            Some(token) => {
                let pos = token.span().start();
                return Err(format!("{}:{} Invalid prefix syntax", pos.line, pos.column).into());
            }
            _ => return Err(" Unexpected termination while parsing prefix".into()),
        },
        _ => String::new(),
    };

    let content = match stream.next() {
        Some(TokenTree::Literal(literal)) => {
            let pos = literal.span().start();
            syn::parse2::<syn::LitStr>(literal.into_token_stream())
                .map_err(|e| {
                    format!(
                        "{}:{} Invalid translation content: {}",
                        pos.line, pos.column, e
                    )
                })?
                .value()
        }
        Some(token) => {
            let pos = token.span().start();
            return Err(format!("{}:{} Invalid translation content", pos.line, pos.column).into());
        }
        _ => return Err(" Unexpected termination while parsing content".into()),
    };

    contents.insert(format!("{} {}", prefix, content));

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use syn::__private::ToTokens;

    use crate::source::extractor::extract_inner;

    #[test]
    fn test_extract_inner() {
        let source = include_str!("../../../example/src/main.rs");

        let stream = syn::parse_file(source).unwrap().into_token_stream();
        let mut contents = HashSet::new();
        let mut locale = String::new();

        extract_inner(stream, &mut contents, &mut locale).unwrap();

        println!("contents: {:#?}", contents);
        println!("locale: {}", locale);
    }
}
