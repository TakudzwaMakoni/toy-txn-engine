use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::{env, process};

use crate::events::ProcessEvent;
use crate::ledger::Ledger;
use crate::record::Record;

pub fn the_app() -> Result<ProcessEvent, Box<dyn Error>> {
    // begin preprocessing
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("usage:\n cargo run -- [transactions file] ");
        process::exit(1);
    }

    let file = File::open(&args[1])?;
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(BufReader::new(file));

    // begin processing
    let mut ledger = Ledger::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        ledger.process_transaction(record)?;
    }

    ledger.print_accounts()?;
    Ok(ProcessEvent::ProcessComplete)
}
