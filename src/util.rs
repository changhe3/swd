use std::{ffi::OsString, fmt::Display, process::Command};

pub(crate) struct OsStrBuf<'a> {
    pub(crate) inner: &'a mut OsString,
}

impl<'a> core::fmt::Write for OsStrBuf<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.inner.push(s);
        Ok(())
    }
}

pub(crate) struct PrettyCmd<'a> {
    cmd: &'a Command,
}

impl<'a> Display for PrettyCmd<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let iter = std::iter::once(self.cmd.get_program()).chain(self.cmd.get_args());
        for arg in iter {
            if let Some(utf8) = arg.to_str() {
                f.write_str(&shlex::quote(utf8))?;
                f.write_str(" ")?;
            } else {
                f.write_fmt(format_args!("{:?} ", arg))?;
            }
        }
        Ok(())
    }
}

impl<'a> PrettyCmd<'a> {
    pub(crate) fn new(cmd: &'a Command) -> Self {
        Self { cmd }
    }
}

#[allow(unused_macros)]
macro_rules! os_str_format {
    ($($arg:tt)*) => {{
        use core::fmt::Write;

        let mut res = std::ffi::OsString::new();
        let mut buf = $crate::util::OsStrBuf {
            inner: &mut res
        };
        core::write!(buf, $($arg)*).unwrap();
        res
    }};
}

#[allow(unused_imports)]
pub(crate) use os_str_format;

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        os_str_format!("hello {} {}", "world", 123);
    }
}
