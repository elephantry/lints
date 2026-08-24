#![allow(dead_code)]

fn main() {}

fn pager_new(elephantry: &elephantry::Connection) -> elephantry::Result<elephantry::Pager<()>> {
    let rows = elephantry.execute("select 1")?;
    let pager = elephantry::Pager::<()>::new(rows.into(), 10, 0, 10);

    Ok(pager)
}
