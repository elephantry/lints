dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Check syntax, without execute its.
    /// ```
    pub INVALID_QUERY,
    Deny,
    "check SQL query"
}

impl<'tcx> rustc_lint::LateLintPass<'tcx> for InvalidQuery {
    fn check_expr(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        expr: &'tcx rustc_hir::Expr<'tcx>,
    ) {
        let config = elephantry::Config::from_env().unwrap();
        let elephantry = elephantry::Pool::from_config(&config).unwrap();

        let rustc_hir::ExprKind::MethodCall(name, recv, args, _) = expr.kind else {
            return;
        };

        let caller_ty = cx.typeck_results().expr_ty(recv);

        let rustc_middle::ty::Ref(_, ty, _) = caller_ty.kind() else {
            return;
        };

        let rustc_middle::ty::Adt(adt, _) = ty.kind() else {
            return;
        };

        let did = adt.did();

        if cx.tcx.crate_name(did.krate).as_str() != "elephantry" {
            return;
        }

        if cx.tcx.item_name(did).as_str() != "Connection" {
            return;
        }

        let result = match name.ident.as_str() {
            "execute" => Self::check_execute(&elephantry, args[0]),
            "query" | "query_one" => Self::check_query(&elephantry, args[0]),
            "find_all" => Self::check_suffix(cx, &elephantry, args[0]),
            "find_where" => {
                Self::check_clause(cx, &elephantry, args[0])
                    .and_then(|_| Self::check_suffix(cx, &elephantry, args[2]))
            }

            _ => return,
        };

        let Err(err) = result else {
            return;
        };

        clippy_utils::diagnostics::span_lint_and_help(
            cx,
            INVALID_QUERY,
            args[0].span,
            "invalid SQL query",
            None,
            err.to_string(),
        );
    }
}

impl InvalidQuery {
    fn check_execute(
        elephantry: &elephantry::Connection,
        arg: rustc_hir::Expr,
    ) -> elephantry::Result {
        let rustc_hir::ExprKind::Lit(lit) = &arg.kind else {
            return Ok(());
        };

        let rustc_ast::LitKind::Str(symbol, _) = lit.node else {
            return Ok(());
        };

        Self::check_sql(elephantry, &symbol.to_ident_string())
    }

    fn check_query(
        elephantry: &elephantry::Connection,
        arg: rustc_hir::Expr,
    ) -> elephantry::Result {
        let rustc_hir::ExprKind::Lit(lit) = &arg.kind else {
            return Ok(());
        };

        let rustc_ast::LitKind::Str(symbol, _) = lit.node else {
            return Ok(());
        };

        let query = symbol.to_ident_string();

        Self::check_sql(elephantry, &Self::order_parameters(&query))
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

    fn check_clause(cx: &rustc_lint::LateContext<'_>, elephantry: &elephantry::Connection, arg: rustc_hir::Expr) -> elephantry::Result {
        let rustc_hir::ExprKind::Lit(lit) = &arg.kind else {
            return Ok(());
        };

        let rustc_ast::LitKind::Str(symbol, _) = lit.node else {
            return Ok(());
        };

        let clause = symbol.to_ident_string();

        Self::check_sql(elephantry, &format!("select 1 where {}", Self::order_parameters(&clause)))
    }

    fn check_suffix(cx: &rustc_lint::LateContext<'_>, elephantry: &elephantry::Connection, suffix: rustc_hir::Expr) -> elephantry::Result {
        let Some(inner) = clippy_utils::as_some_expr(cx, &suffix) else {
            return Ok(());
        };

        let rustc_hir::ExprKind::Lit(lit) = inner.kind else {
            return Ok(());
        };

        let rustc_ast::LitKind::Str(symbol, _) = lit.node else {
            return Ok(());
        };

        Self::check_sql(elephantry, &format!("select 1 {}", symbol.to_ident_string()))
    }

    fn check_sql(elephantry: &elephantry::Connection, query: &str) -> elephantry::Result {
        let mut query = query.to_string();
        if !query.ends_with(';') {
            query.push(';');
        }

        let sql = format!("DO $TEST$ BEGIN RETURN;{query}END; $TEST$;");

        elephantry.execute(&sql).map(|_| ())
    }
}
