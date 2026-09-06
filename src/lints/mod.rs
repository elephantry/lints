pub mod invalid_query;
pub mod mix_param_type;
pub mod pager_start_page;
pub mod param_arg_count;

#[derive(Debug)]
struct Method<'a> {
    path: String,
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
        && let rustc_middle::ty::Adt(adt, _) = ty.kind()
    {
        adt
    } else if let rustc_middle::ty::Adt(adt, _) = caller_ty.kind() {
        adt
    } else {
        return None;
    };

    let did = adt.did();

    let method = Method {
        path: cx.tcx.def_path_str(did),
        name: name.ident.to_string(),
        args,
    };

    Some(method)
}

#[derive(Debug)]
struct Function<'a> {
    path: String,
    args: &'a [rustc_hir::Expr<'a>],
}

fn function_call<'a>(
    cx: &rustc_lint::LateContext<'_>,
    expr: &rustc_hir::Expr<'a>,
) -> Option<Function<'a>> {
    let rustc_hir::ExprKind::Call(function, args) = expr.kind else {
        return None;
    };

    let caller_ty = cx.typeck_results().expr_ty(function);

    let rustc_middle::ty::FnDef(ty, _) = caller_ty.kind() else {
        return None;
    };

    let function = Function {
        path: cx.tcx.def_path_str(*ty),
        args,
    };

    Some(function)
}

fn expr_to_string(expr: &rustc_hir::Expr<'_>) -> Option<String> {
    let rustc_hir::ExprKind::Lit(lit) = &expr.kind else {
        return None;
    };

    let rustc_ast::LitKind::Str(symbol, _) = lit.node else {
        return None;
    };

    Some(symbol.to_ident_string())
}
