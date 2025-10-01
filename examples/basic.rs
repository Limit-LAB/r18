r18::init!("examples/tr");

fn main() {
    let name = "ho-229";

    r18::auto_detect!();
    println!("{}", r18::tr!("Hello, {}", name));
}
