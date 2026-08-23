#[allow(dead_code)]

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
    #[derive(elephantry::Entity)]
    #[elephantry(model = "Model", structure  = "Structure")]
    struct Entity {}

    elephantry.find_all::<Model>(Some("order y 1")).map(convert::unit)
}

mod convert {
    pub fn unit<T>(_: T) -> () {
        ()
    }
}
