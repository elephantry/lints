mod invalid_query;
mod pager_start_page;

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    lint_store.register_lints(&[
        invalid_query::INVALID_QUERY,
        pager_start_page::PAGER_START_PAGE,
    ]);
    lint_store.register_late_pass(|_| Box::new(invalid_query::InvalidQuery));
    lint_store.register_late_pass(|_| Box::new(pager_start_page::PagerStartPage));
}

#[derive(Debug)]
struct Method<'a> {
    krate: String,
    path: String,
    caller: String,
    name: String,
    args: &'a [rustc_hir::Expr<'a>],
}

fn method_call<'a>(
    cx: &rustc_lint::LateContext<'_>,
    expr: &'a rustc_hir::Expr<'_>,
) -> Option<Method<'a>> {
    let rustc_hir::ExprKind::MethodCall(name, recv, args, _) = expr.kind else {
        return None;
    };

    let caller_ty = cx.typeck_results().expr_ty(recv);

    let adt = if let rustc_middle::ty::Ref(_, ty, _) = caller_ty.kind()
        && let rustc_middle::ty::Adt(adt, _) = ty.kind() {
        adt
    } else if let rustc_middle::ty::Adt(adt, _) = caller_ty.kind() {
        adt
    } else {
        return None;
    };

    let did = adt.did();

    let method = Method {
        krate: cx.tcx.crate_name(did.krate).to_string(),
        path: cx.tcx.def_path_str(did).to_string(),
        caller: cx.tcx.item_name(did).to_string(),
        name: name.ident.to_string(),
        args,
    };

    Some(method)
}

#[derive(Debug)]
struct Function<'a> {
    krate: String,
    path: String,
    name: String,
    args: &'a [rustc_hir::Expr<'a>],
}

fn function_call<'a>(
    cx: &rustc_lint::LateContext<'_>,
    expr: &'a rustc_hir::Expr<'_>,
) -> Option<Function<'a>> {
    let rustc_hir::ExprKind::Call(function, args) = expr.kind else {
        return None;
    };

    let caller_ty = cx.typeck_results().expr_ty(function);

    let rustc_middle::ty::FnDef(ty, _) = caller_ty.kind() else {
        return None;
    };

    let function = Function {
        krate: cx.tcx.crate_name(ty.krate).to_string(),
        path: cx.tcx.def_path_str(*ty).to_string(),
        name: cx.tcx.item_name(*ty).to_string(),
        args,
    };

    Some(function)
}
