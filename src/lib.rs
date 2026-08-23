#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;

mod lints;

#[test]
fn ui() {
    dylint_testing::ui::Test::examples(env!("CARGO_PKG_NAME"))
        .rustc_flags(["--test"])
        .run();
}
