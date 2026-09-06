#![allow(dead_code)]

fn main() {}

fn fail(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.query_one::<()>("select $1, $*", &[&1])
}
