pub type Result<'a, T = ()> = std::result::Result<T, Error<'a>>;

#[derive(Debug)]
pub struct Error<'e> {
    pub error: elephantry::Error,
    pub expr: &'e rustc_hir::Expr<'e>,
    pub sql: String,
}

impl Error<'_> {
    pub fn span(&self) -> rustc_span::Span {
        self.expr.span
    }
}

impl std::error::Error for Error<'_> {}

impl std::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let elephantry::Error::Sql(error) = &self.error else {
            return write!(f, "{}", self.error);
        };

        static REGEX: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new("(\\d+)\n$").unwrap());

        let mut n = 0;
        let message = error.error_message().unwrap().unwrap_or_default();
        let message = REGEX.replace(&message, |captures: &regex::Captures<'_>| {
            n = captures[1].parse::<usize>().unwrap() - 24;

            format!("{n}:\n\n")
        });

        f.write_str(&message)?;
        f.write_str(&self.sql)?;
        write!(f, "\n{:_>1$}", '^', n + 1)
    }
}

impl<'e> From<(elephantry::Error, &'e rustc_hir::Expr<'e>, String)> for Error<'e> {
    fn from(value: (elephantry::Error, &'e rustc_hir::Expr<'e>, String)) -> Self {
        Self {
            error: value.0,
            expr: value.1,
            sql: value.2,
        }
    }
}
