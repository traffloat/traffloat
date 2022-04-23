use std::{fmt, io};

pub trait BoxContext<T> {
    fn context<C: fmt::Display + Send + Sync + 'static>(self, context: C) -> anyhow::Result<T>;
    fn with_context<C: fmt::Display + Send + Sync + 'static, F: FnOnce() -> C>(
        self,
        f: F,
    ) -> anyhow::Result<T>;
}

impl<T> BoxContext<T> for Result<T, Box<dyn std::error::Error + 'static>> {
    fn context<C: fmt::Display + Send + Sync + 'static>(self, context: C) -> anyhow::Result<T> {
        anyhow::Context::context(
            self.map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string())),
            context,
        )
    }
    fn with_context<C: fmt::Display + Send + Sync + 'static, F: FnOnce() -> C>(
        self,
        f: F,
    ) -> anyhow::Result<T> {
        anyhow::Context::with_context(
            self.map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string())),
            f,
        )
    }
}
