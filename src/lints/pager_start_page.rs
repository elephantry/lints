rustc_session::declare_lint!(pub PAGER_START_PAGE, Warn, "check pager start page");
rustc_session::declare_lint_pass!(PagerStartPage => [PAGER_START_PAGE]);

impl<'tcx> rustc_lint::LateLintPass<'tcx> for PagerStartPage {
    fn check_expr(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        expr: &'tcx rustc_hir::Expr<'tcx>,
    ) {
        if let Some(function) = super::function_call(cx, expr) {
            if let Err(expr) = Self::check_pager_new(&function) {
                clippy_utils::diagnostics::span_lint(
                    cx,
                    PAGER_START_PAGE,
                    expr.span,
                    "pager page starts at 1",
                );
            }
        }

        if let Some(method) = super::method_call(cx, expr) {
            if let Err(expr) = Self::check_paginate_find_where(&method) {
                clippy_utils::diagnostics::span_lint(
                    cx,
                    PAGER_START_PAGE,
                    expr.span,
                    "pager page starts at 1",
                );
            }
        }
    }
}

impl PagerStartPage {
    fn check_paginate_find_where<'a>(
        method: &super::Method<'a>,
    ) -> Result<(), rustc_hir::Expr<'a>> {
        if method.path != "elephantry::Connection" {
            return Ok(());
        }

        if method.name != "paginate_find_where" {
            return Ok(());
        }

        let arg = method.args[3];

        let rustc_hir::ExprKind::Lit(lit) = arg.kind else {
            return Ok(());
        };

        let rustc_ast::LitKind::Int(symbol, _) = lit.node else {
            return Ok(());
        };

        if symbol.get() >= 1 { Ok(()) } else { Err(arg) }
    }

    fn check_pager_new<'a>(function: &super::Function<'a>) -> Result<(), rustc_hir::Expr<'a>> {
        if function.path != "elephantry::Pager::<E>::new" {
            return Ok(());
        }

        let arg = function.args[2];

        let rustc_hir::ExprKind::Lit(lit) = arg.kind else {
            return Ok(());
        };

        let rustc_ast::LitKind::Int(symbol, _) = lit.node else {
            return Ok(());
        };

        if symbol.get() >= 1 { Ok(()) } else { Err(arg) }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn pager_start_page() {
        dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "pager_start_page");
    }
}
