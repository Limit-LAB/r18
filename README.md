# 🔞

[![crate.io](https://img.shields.io/crates/v/r18.svg)](https://crates.io/crates/r18)
[![docs](https://docs.rs/r18/badge.svg)](https://docs.rs/r18)
[![build](https://github.com/Limit-IM/r18/actions/workflows/rust.yml/badge.svg)](https://github.com/Limit-IM/r18/actions/workflows/rust.yml)

`r18` is a crate intends to simplify the internationalisation of Rust projects.

## Usage

Add `r18` as your project dependency:

```toml
[dependencies]
r18 = "*"
```

Create a `JSON` translation file whose filename follows [IETF BCP 47](https://www.wikiwand.com/en/IETF_language_tag) language tag, like below
(you can generate it by [CLI Tool](#cli-tool-usage)):

```json
// PATH: ./tr/zh-CN.json
{
    "Hello, {}": "你好，{}"
}
```

Then add `r18::init!` to the global scope of your code with the directory where translation files in (in following example is `./tr`).

```rust
r18::init!("tr");
```

After initialising the `r18`, use `auto_detect!` to detect locale and load translation model automatically.  
If you want, you can use `set_locale!` to set locale manually.  
After above process, use `tr!` to get your text which has been translated.

```rust
r18::init!("tr");

fn main() {
    r18::auto_detect!(); // get locale & set

    let name = "ho-229";
    println!("{}", tr!("Hello, {}", name));

    // reset locale to disable translation
    r18::set_locale!("");
    assert_eq!("Hello, ho-229", tr!("Hello, {}", name));
}
```

You can find a complete example [here](./example/). You can run the example with following command:

```shell
cargo run -p example
```

## CLI Tool Usage

Run the below command to install `cargo r18`:

```shell
cargo install cargo-r18
```

After creating the translation directory and writing code as above, you can run the following command to
generate translation files (eg. TODO.zh-CN.json):

```shell
cargo r18 generate zh-CN
```

Additionally, you can generate todo files of untranslated texts after changing your source by:

```shell
cargo r18 update
```

***LIMITATION:*** `cargo r18` is only scanning macros named `init` and `tr` that it can NOT recognise which belong to `r18` or not,
you should make sure that no similar macros are named in your source before using `cargo r18`.

Run `cargo r18 -h` for more options.

## Credit

* [rust-i18n](https://github.com/longbridgeapp/rust-i18n)
