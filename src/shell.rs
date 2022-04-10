pub(crate) fn status(prefix: &str, msg: &str, is_indented: bool) {
    if is_indented {
        println!("    {prefix} {msg}");
    } else {
        println!("{prefix} {msg}");
    }
}
