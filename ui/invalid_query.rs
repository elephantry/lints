fn main() -> elephantry::Result {
    let config = elephantry::Config::from_env()?;
    let elephantry = elephantry::Pool::from_config(&config)?;

    pass(&elephantry)?;
    fail(&elephantry)?;
    query(&elephantry)?;

    Ok(())
}

fn pass(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("select 1").map(convert::unit)
}

fn fail(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.execute("selec 1").map(convert::unit)
}

fn query(elephantry: &elephantry::Connection) -> elephantry::Result {
    elephantry.query::<()>("select $*", &[&1]).map(|_| ())
}

mod convert {
    pub fn unit<T>(_: T) -> () {
        ()
    }
}
