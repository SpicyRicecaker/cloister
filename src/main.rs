use anyhow::Error;
use cloister::{init_logger, App};

fn main() -> Result<(), Error> {
    init_logger();

    let app = App::new()?;
    app.run()?;

    Ok(())
}
