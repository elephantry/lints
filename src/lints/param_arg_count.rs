rustc_session::declare_lint!(pub PARAM_ARG_COUNT, Deny, "check if params arg count");
rustc_session::declare_lint_pass!(ParamArgCount => [PARAM_ARG_COUNT]);

pub fn register_lint(lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[PARAM_ARG_COUNT]);
}

pub fn register_pass(lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_late_pass(|_| Box::new(ParamArgCount));
}

enum Error<'m> {
    NeverUse(&'m rustc_hir::Expr<'m>, &'m [rustc_hir::Expr<'m>]),
    Missing(&'m super::Method<'m>, usize, usize),
}

impl Error<'_> {
    fn diag(&self, diag: &mut rustc_errors::Diag<()>) {
        match self {
            Self::NeverUse(query, args) => {
                diag.primary_message("arguments never used");
                diag.span_label(query.span, "parameter missing");

                for arg in *args {
                    diag.span_label(arg.span, "argument never used");
                }
            }
            Self::Missing(method, expected, actual) => {
                diag.span(vec![method.args[0].span, method.args[1].span]);
                diag.primary_message(format!(
                    "{expected} positional argument(s) in query, but there is {actual} argument(s)"
                ));
            }
        }
    }
}

impl<'tcx> rustc_lint::LateLintPass<'tcx> for ParamArgCount {
    fn check_expr(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        expr: &'tcx rustc_hir::Expr<'tcx>,
    ) {
        let Some(method) = super::method_call(cx, expr) else {
            return;
        };

        if let Err(err) = Self::check(&method) {
            clippy_utils::diagnostics::span_lint_and_then(
                cx,
                PARAM_ARG_COUNT,
                method.args[0].span,
                "invalid number of param",
                |diag| {
                    err.diag(diag);
                },
            );
        }
    }
}

impl ParamArgCount {
    fn check<'a>(method: &'a super::Method<'a>) -> std::result::Result<(), Error<'a>> {
        if !matches!(
            method.path.as_str(),
            "elephantry::Connection" | "elephantry::Async"
        ) {
            return Ok(());
        }

        if !matches!(method.name.as_str(), "query" | "query_one")
            && !method.name.ends_with("_where")
        {
            return Ok(());
        }

        let Some(query) = super::expr_to_string(&method.args[0]) else {
            return Ok(());
        };

        let rustc_hir::ExprKind::AddrOf(_, _, r#ref) = method.args[1].kind else {
            return Ok(());
        };

        let rustc_hir::ExprKind::Array(params) = r#ref.kind else {
            return Ok(());
        };

        let expected = Self::count_param(&query);
        let actual = params.len();

        if expected == actual {
            return Ok(());
        }

        if expected == 0 && actual > 0 {
            return Err(Error::NeverUse(&method.args[0], params));
        }

        Err(Error::Missing(method, expected, actual))
    }

    fn count_param(query: &str) -> usize {
        let query = if query.contains("$*") {
            static REGEX: std::sync::LazyLock<regex::Regex> =
                std::sync::LazyLock::new(|| regex::Regex::new(r"\$\*").unwrap());

            let mut count = 0;

            REGEX.replace_all(query, |captures: &regex::Captures<'_>| {
                count += 1;

                captures[0].replace("$*", &format!("${count}"))
            }).to_string()
        } else {
            query.to_string()
        };

        static REGEX: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new(r"\$(\d+)").unwrap());

        REGEX.captures_iter(&query)
            .map(|x| x[1].parse().unwrap())
            .max()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn param_arg_count() {
        dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "param_arg_count");
    }
}
