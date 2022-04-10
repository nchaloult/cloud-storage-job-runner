pub(crate) fn status(prefix: Option<&str>, msg: &str) {
    match prefix {
        Some(p) => println!("{p} {msg}"),
        None => println!("{msg}"),
    }
}
