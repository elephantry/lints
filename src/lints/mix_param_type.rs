rustc_session::declare_lint!(pub MIX_PARAM_TYPE, Warn, "check ordered and unordered param mix");
rustc_session::declare_lint_pass!(MixParamType => [MIX_PARAM_TYPE]);

pub fn register_lint(lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[MIX_PARAM_TYPE]);
}

pub fn register_pass(lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_late_pass(|_| Box::new(MixParamType));
}


impl<'tcx> rustc_lint::LateLintPass<'tcx> for MixParamType {
    fn check_expr(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        expr: &'tcx rustc_hir::Expr<'tcx>,
    ) {
        let Some(method) = super::method_call(cx, expr) else {
            return;
        };

        if Self::check(&method).is_err() {
            clippy_utils::diagnostics::span_lint(
                cx,
                MIX_PARAM_TYPE,
                method.args[0].span,
                "this query contains both ordered and unordered param",
            );
        }
    }
}

impl MixParamType {
    fn check<'a>(method: &'a super::Method<'a>) -> Result<(), ()> {
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

        if !query.contains("$*") {
            return Ok(());
        }

        static REGEX: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new(r"\$(\d+)").unwrap());

        if REGEX.is_match(&query) {
            Err(())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn mix_param_type() {
        dylint_testing::ui_test_example(env!("CARGO_PKG_NAME"), "mix_param_type");
    }
}
