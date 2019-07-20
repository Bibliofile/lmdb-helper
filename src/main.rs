extern crate lmdb_zero as lmdb;
extern crate clap;
use clap::{Arg, App};

use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("lmdb-helper")
        .version("0.1")
        .author("Bibliofile <bibliofilegit@gmail.com>")
        .about("Simple tool to extract information about LMDB databases")
        .arg(Arg::with_name("database")
            .short("d")
            .long("database")
            .value_name("DATABASE")
            .help("The database to read, if no database is passed and the lmdb uses named databases, the returned keys will be the possible database names")
            .takes_value(true))
        .arg(Arg::with_name("list")
            .short("l")
            .long("list")
            .help("If passed, only print the possible keys in the database, defaults to true if --extract is not passed")
            .takes_value(false))
        .arg(Arg::with_name("extract")
            .short("e")
            .long("extract")
            .help("If passed, extract the value in the database to a file as specified by --out")
            .takes_value(true))
        .arg(Arg::with_name("out")
            .short("o")
            .long("out")
            .help("Specify the name of the extracted file, defaults to <key>.bin if not specified.")
            .takes_value(true))
        .arg(Arg::with_name("DIR")
            .help("Sets the database directory to use, defaults to the current working directory.")
            .required(false)
            .index(1))
        .get_matches();

    let dir = matches.value_of("DIR").unwrap_or(".");
    let dir = dir.replace(r"\", "/");
    let dir = dir.trim_end_matches("/");

    let env = unsafe {
        let mut env = lmdb::EnvBuilder::new().unwrap();
        env.set_maxdbs(16).unwrap();
        env.open(dir, lmdb::open::RDONLY, 0o600)
    }?;
    let db = lmdb::Database::open(&env, matches.value_of("database"), &lmdb::DatabaseOptions::defaults())?;

    if matches.is_present("list") || matches.value_of("extract").is_none() {
        print_list(&db)?;
    }

    if let Some(key) = matches.value_of("extract") {
        extract_key(&db, key, matches.value_of("out").unwrap_or(&format!("{}.bin", key)))?;
    }

    Ok(())
}

/// Prints the keys in a database and the size of the values.
fn print_list(db: &lmdb::Database) -> Result<(), lmdb::Error> {
    let txn = lmdb::ReadTransaction::new(db.env())?;
    let access = txn.access();
    let mut cursor = txn.cursor(db)?;
    let mut iter = lmdb::CursorIter::new(
        lmdb::MaybeOwned::Borrowed(&mut cursor),
        &access,
        |c, a| c.first(a),
        lmdb::Cursor::next::<str, [u8]>
    )?;

    let mut key_width = 3;
    let mut val_width = 12;

    let mut entries = Vec::new();
    while let Some(Ok((key, val))) = iter.next() {
        entries.push((key, val.len()));
        key_width = std::cmp::max(key_width, key.len());
        val_width = std::cmp::max(val_width, format!("{}", val.len()).len());
    }
    println!("| {:key_width$} | {:val_width$} |", "Key", "Size (bytes)", key_width=key_width, val_width=val_width);
    println!("| {} | {} |", "-".repeat(key_width), "-".repeat(val_width));
    for (key, len) in entries {
        println!("| {:key_width$} | {:<val_width$} |", key, len, key_width=key_width, val_width=val_width);
    }
    Ok(())
}

fn extract_key(db: &lmdb::Database, key: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let txn = lmdb::ReadTransaction::new(db.env())?;
    let access = txn.access();
    let data: &[u8] = access.get(&db, key)?;

    let mut file = File::create(out_file)?;
    file.write_all(data)?;

    Ok(())
}
