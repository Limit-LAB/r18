r18::init!("tr");

fn main() {
    let name = "ho-229";
    r18::set_locale!("");
    assert_eq!(format!("Hello, {}", name), r18::tr!("Hello, {}", name));

    r18::set_locale!("zh_CN");
    assert_eq!(format!("你好，{}", name), r18::tr!("Hello, {}", name));
    assert_eq!(
        r#"调试：Custom { kind: Other, error: "An error" }"#,
        r18::tr!(
            "Debug: {}",
            format!(
                "{:?}",
                std::io::Error::new(std::io::ErrorKind::Other, "An error")
            )
        )
    );

    r18::auto_detect!();
    println!("{}", r18::tr!("Hello, {}", name));
}
