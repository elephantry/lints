rustc_session::declare_lint!(pub INVALID_QUERY, Deny, "check SQL query");
rustc_session::declare_lint_pass!(InvalidQuery => [INVALID_QUERY]);

impl<'tcx> rustc_lint::LateLintPass<'tcx> for InvalidQuery {
    fn check_expr(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        expr: &'tcx rustc_hir::Expr<'tcx>,
    ) {
        let Some(method) = super::method_call(cx, expr) else {
            return;
        };

        let config = elephantry::Config::from_env().unwrap();
        let elephantry = elephantry::Pool::from_config(&config).unwrap();

        let result = Self::check_execute(&elephantry, &method)
            .and_then(|_| Self::check_query(&elephantry, &method))
            .and_then(|_| Self::check_suffix(cx, &elephantry, &method))
            .and_then(|_| Self::check_clause(&elephantry, &method));

        let Err((err, expr)) = result else {
            return;
        };

        clippy_utils::diagnostics::span_lint_and_help(
            cx,
            INVALID_QUERY,
            expr.span,
            "invalid SQL query",
            None,
            err.to_string(),
        );
    }
}

type Result<'a, T = ()> = std::result::Result<T, (elephantry::Error, &'a rustc_hir::Expr<'a>)>;

impl InvalidQuery {
    fn check_execute<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        if method.name != "execute" {
            return Ok(());
        }

        Self::check_sql(elephantry, None, &method.args[0], false)
    }

    fn check_query<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        if method.name != "query" || method.name != "query_one" {
            return Ok(());
        }

        Self::check_sql(elephantry, None, &method.args[0], true)
    }

    fn check_clause<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        if !method.name.ends_with("_where") {
            return Ok(());
        }

        Self::check_sql(elephantry, Some("select 1 where"), &method.args[0], true)
    }

    fn check_suffix<'a>(
        cx: &rustc_lint::LateContext<'_>,
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        let arg = match method.name.as_str() {
            "find_all" => &method.args[0],
            "find_where" | "paginate_find_where" => &method.args[2],
            _ => return Ok(()),
        };

        let Some(suffix) = clippy_utils::as_some_expr(cx, &arg) else {
            return Ok(());
        };

        Self::check_sql(elephantry, Some("select 1"), suffix, false)
    }

    fn check_sql<'a>(
        elephantry: &elephantry::Connection,
        prefix: Option<&str>,
        arg: &'a rustc_hir::Expr<'a>,
        order: bool,
    ) -> Result<'a> {
        let rustc_hir::ExprKind::Lit(lit) = &arg.kind else {
            return Ok(());
        };

        let rustc_ast::LitKind::Str(symbol, _) = lit.node else {
            return Ok(());
        };

        let query = symbol.to_ident_string();

        let mut query = if order {
            Self::order_parameters(&query).to_string()
        } else {
            query.to_string()
        };

        if !query.ends_with(';') {
            query.push(';');
        }

        let sql = format!(
            "DO $TEST$ BEGIN RETURN;{} {query}END; $TEST$;",
            prefix.unwrap_or_default()
        );

        elephantry.execute(&sql).map(|_| ()).map_err(|e| (e, arg))
    }

    fn order_parameters(query: &str) -> std::borrow::Cow<'_, str> {
        static REGEX: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new(r"\$\*").unwrap());

        let mut count = 0;

        REGEX.replace_all(query, |captures: &regex::Captures<'_>| {
            count += 1;

            captures[0].replace("$*", &format!("${count}"))
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn invalid_query() {
        dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "invalid_query");
    }
}
