#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

dylint_linting::dylint_library!();

mod lints;
mod result;

use result::*;

#[unsafe(no_mangle)]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    macro_rules! register {
        ($name:ident) => {
            lints::$name::register_lint(lint_store);
            lints::$name::register_pass(lint_store);
        };
    }

    dylint_linting::init_config(sess);

    register!(invalid_query);
    register!(pager_start_page);
    register!(param_arg_count);
}
