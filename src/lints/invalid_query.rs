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

        let results = [
            Self::check_execute(&elephantry, &method),
            Self::check_query(&elephantry, &method),
            Self::check_suffix(cx, &elephantry, &method),
            Self::check_clause(&elephantry, &method),
            Self::check_upsert_target(&elephantry, &method),
            Self::check_upsert_action(&elephantry, &method),
        ];

        results
            .into_iter()
            .filter_map(std::result::Result::err)
            .for_each(|e| {
                clippy_utils::diagnostics::span_lint_and_help(
                    cx,
                    INVALID_QUERY,
                    e.span(),
                    "invalid SQL query",
                    None,
                    e.to_string(),
                );
            });
    }
}

impl InvalidQuery {
    fn check_execute<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> crate::Result<'a> {
        if !matches!(
            method.path.as_str(),
            "elephantry::Connection" | "elephantry::Async"
        ) {
            return Ok(());
        }

        if method.name != "execute" {
            return Ok(());
        }

        Self::check_expr(elephantry, None, &method.args[0], false)
    }

    fn check_query<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> crate::Result<'a> {
        if !matches!(
            method.path.as_str(),
            "elephantry::Connection" | "elephantry::Async"
        ) {
            return Ok(());
        }

        if method.name != "query" || method.name != "query_one" {
            return Ok(());
        }

        Self::check_expr(elephantry, None, &method.args[0], true)
    }

    fn check_clause<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> crate::Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        if !method.name.ends_with("_where") {
            return Ok(());
        }

        Self::check_expr(elephantry, Some("select 1 where"), &method.args[0], true)
    }

    fn check_suffix<'a>(
        cx: &rustc_lint::LateContext<'_>,
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> crate::Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        let arg = match method.name.as_str() {
            "find_all" => &method.args[0],
            "find_where" | "paginate_find_where" => &method.args[2],
            _ => return Ok(()),
        };

        let Some(suffix) = clippy_utils::as_some_expr(cx, arg) else {
            return Ok(());
        };

        Self::check_expr(elephantry, Some("select 1"), suffix, false)
    }

    fn check_upsert_target<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> crate::Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        if method.name != "upsert_one" {
            return Ok(());
        }

        let Some(arg) = super::expr_to_string(&method.args[1]) else {
            return Ok(());
        };

        let query = format!("insert into test values(1) on conflict {arg} do nothing");
        Self::check_sql(elephantry, &query, false).map_err(|e| (e, &method.args[1], query).into())
    }

    fn check_upsert_action<'a>(
        elephantry: &elephantry::Connection,
        method: &super::Method<'a>,
    ) -> crate::Result<'a> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        if method.name != "upsert_one" {
            return Ok(());
        }

        let Some(arg) = super::expr_to_string(&method.args[2]) else {
            return Ok(());
        };

        let query = format!("insert into test values(1) on conflict (test) do {arg}");
        Self::check_sql(elephantry, &query, false).map_err(|e| (e, &method.args[2], query).into())
    }

    fn check_expr<'a>(
        elephantry: &elephantry::Connection,
        prefix: Option<&str>,
        arg: &'a rustc_hir::Expr<'a>,
        order: bool,
    ) -> crate::Result<'a> {
        let Some(s) = super::expr_to_string(arg) else {
            return Ok(());
        };

        let query = format!("{} {s}", prefix.unwrap_or_default());
        Self::check_sql(elephantry, &query, order).map_err(|e| (e, arg, query).into())
    }

    fn check_sql(
        elephantry: &elephantry::Connection,
        sql: &str,
        order: bool,
    ) -> elephantry::Result {
        let mut query = if order {
            Self::order_parameters(sql).to_string()
        } else {
            sql.to_string()
        };

        if !query.ends_with(';') {
            query.push(';');
        }

        let query = format!("DO $TEST$ BEGIN RETURN;{query}END; $TEST$;");

        elephantry.execute(&query).map(|_| ())
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
