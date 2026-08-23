#[allow(dead_code)]

#[derive(elephantry::Entity)]
#[elephantry(model = "Model", structure  = "Structure")]
struct Entity {}

fn main() {}

fn pass(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("select 1").map(convert::unit)
}

fn fail(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("selec 1").map(convert::unit)
}

fn query(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.query::<()>("select $*", &[&1]).map(convert::unit)
}

fn find_all(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.find_all::<Model>(Some("order y 1")).map(convert::unit)
}

fn find_where(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.find_where::<Model>("$* =", &[], None).map(convert::unit)
}

mod convert {
    pub fn unit<T>(_: T) -> () {
        ()
    }
}
