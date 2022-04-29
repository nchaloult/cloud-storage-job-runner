use std::{
    fmt,
    io::{self, Write},
};

use termcolor::{
    Color::{Green, Red},
    ColorChoice, ColorSpec, StandardStream, WriteColor,
};

pub fn status(prefix: &str, msg: &dyn fmt::Display, is_indented: bool) -> io::Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    stderr.reset()?; // Just in case.
    stderr.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Green)))?;
    if is_indented {
        write!(stderr, "    {prefix}")?;
    } else {
        write!(stderr, "{prefix}")?;
    }
    stderr.reset()?;
    writeln!(stderr, " {msg}")
}

pub fn error(msg: &dyn fmt::Display) -> io::Result<()> {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    stderr.reset()?; // Just in case.
    stderr.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Red)))?;
    write!(stderr, "error: ")?;
    stderr.reset()?;
    writeln!(stderr, "{msg}")
}
