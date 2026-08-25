#![allow(dead_code)]

fn main() {}

fn pass(elephantry: &elephantry::Connection) -> elephantry::Result<()> {
    elephantry.query_one("select $*", &[&1])
}

fn fail(elephantry: &elephantry::Connection) -> elephantry::Result<()> {
    elephantry.query_one::<()>("select $*", &[])?;
    elephantry.query_one::<()>("select $1, $2", &[&1, &2, &3])?;
    elephantry.query_one::<()>("select 1", &[&1, &2])
}
