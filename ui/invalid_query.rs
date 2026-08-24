#![allow(dead_code)]

#[derive(Default, elephantry::Entity)]
#[elephantry(model = "Model", structure = "Structure")]
struct Entity {}

fn main() {}

fn pass(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("select 1").map(convert::unit)
}

fn fail(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("selec 1").map(convert::unit)
}

fn query(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry
        .query::<()>("select $*", &[&1])
        .map(convert::unit)
}

fn find_all(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry
        .find_all::<Model>(Some("order y 1"))
        .map(convert::unit)
}

fn find_where(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry
        .find_where::<Model>("$* =", &[], None)
        .map(convert::unit)
}

fn paginate_find_where(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry
        .paginate_find_where::<Model>("$* = 0", &[], 20, 0, None)
        .map(convert::unit)
}

fn count_where(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry
        .count_where::<Model>("valid = $", &[&true])
        .map(convert::unit)
}

fn exist_where(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry
        .exist_where::<Model>("valid = $", &[&true])
        .map(convert::unit)
}

fn delete_where(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.delete_where::<Model>("", &[]).map(convert::unit)
}

async fn r#async(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.r#async().execute("s").await.map(convert::unit)
}

fn upsert(elephantry: &elephantry::Connection) -> elephantry::Result {
    let entity = Entity::default();

    elephantry
        .upsert_one::<Model>(&entity, "invalid_target", "invalid_action")
        .map(convert::unit)
}

mod convert {
    pub fn unit<T>(_: T) -> () {
        ()
    }
}
