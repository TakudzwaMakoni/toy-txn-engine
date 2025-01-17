use std::process;

use toy_txn_engine::{application::the_app, events::ProcessEvent};

fn main() {
    match the_app() {
        Ok(ProcessEvent::ProcessComplete) => {}
        Ok(ProcessEvent::ExternalErr(err)) => {
            println!("App failed during process: {err}");
            process::exit(1);
        }
        Err(err) => println!("{err:?}"),
    }
}
