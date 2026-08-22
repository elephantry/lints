fn main() -> elephantry::Result {
    let config = elephantry::Config::from_env()?;
    let elephantry = elephantry::Pool::from_config(&config)?;

    pass(&elephantry)?;
    fail(&elephantry)?;

    Ok(())
}

fn pass(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("select 1")
        .map(convert::unit)
}

fn fail(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("selec 1")
        .map(convert::unit)
}

mod convert {
    pub fn unit<T>(_: T) -> () {
        ()
    }
}
