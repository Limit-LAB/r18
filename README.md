# 🔞

[![crate.io](https://img.shields.io/crates/v/r18.svg)](https://crates.io/crates/r18)
[![docs](https://docs.rs/r18/badge.svg)](https://docs.rs/r18)
[![build](https://github.com/Limit-IM/r18/actions/workflows/rust.yml/badge.svg)](https://github.com/Limit-IM/r18/actions/workflows/rust.yml)

`r18` is a crate for internationalising Rust projects.

## Usage

Add `r18` as your project dependency.

```toml
[dependencies]
r18 = "0.1"
```

Create a `JSON` translation file with name `BCP 47` language tag as naming format, like below:

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
    r18::auto_detect!();

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

## Credit

* [rust-i18n](https://github.com/longbridgeapp/rust-i18n)
