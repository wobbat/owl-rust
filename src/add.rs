pub fn run(items: &[String]) {
    for it in items {
        println!("{}", crate::colo::yellow(&format!("added {}", it)));
    }
}