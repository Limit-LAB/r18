r18::init!("tr");

fn main() {
    let name = "ho-229";

    r18::auto_detect!();
    println!("{}", r18::tr!("Hello, {}", name));
}

#[cfg(test)]
mod tests {
    #[test]
    fn functional_test() {
        let name = "ho-229";

        r18::set_locale!("");
        assert_eq!(None, r18::locale!());
        assert_eq!(format!("Hello, {}", name), r18::tr!("Hello, {}", name));

        r18::set_locale!("zh-CN");
        assert_eq!(Some("zh-CN"), r18::locale!());
        assert_eq!(format!("你好，{}", name), r18::tr!("Hello, {}", name));
        assert_eq!(
            format!(
                "要到年底了，我希望你能加把劲，你看隔壁组的 {}，39度羊都是在办公室打地铺的",
                name
            ),
            r18::tr!([".pua"] "Hello, {}", name)
        );
        assert_eq!(
            format!(
                "调试：{:?}",
                std::io::Error::new(std::io::ErrorKind::Other, "An error")
            ),
            r18::tr!(
                "Debug: {}",
                format!(
                    "{:?}",
                    std::io::Error::new(std::io::ErrorKind::Other, "An error")
                )
            )
        );
    }
}
