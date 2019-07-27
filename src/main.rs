extern crate lmdb_zero as lmdb;
extern crate clap;
use clap::{Arg, App};

use std::fmt;
use std::fs::File;
use std::io::prelude::*;

struct PresentableError(&'static str);
impl fmt::Debug for PresentableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Error: {}", self.0)
    }
}

struct Config<'a> {
    database_dir: &'a str,
    database: Option<&'a str>,
    list: bool,
    extract: Option<&'a str>,
    extract_all: bool,
    out_file: Option<&'a str>
}

fn main() -> Result<(), PresentableError> {
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
        .arg(Arg::with_name("extract-all")
            .long("extract-all")
            .help("If passed, extract all keys from a database as <key>.bin")
            .takes_value(false))
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

    let config = Config {
        database_dir: dir,
        database: matches.value_of("database"),
        list: matches.is_present("list") || matches.value_of("extract").is_none(),
        extract: matches.value_of("extract"),
        extract_all: matches.is_present("extract-all"),
        out_file: matches.value_of("out"),
    };

    run(&config)
}

fn run(config: &Config) -> Result<(), PresentableError> {
    let db = get_database(config.database_dir, config.database)?;

    if config.list {
        print_list(&db)?;
    }

    if let Some(key) = config.extract {
        extract_key(&db, key, config.out_file.unwrap_or(&format!("{}.bin", key)))?;
    }

    if config.extract_all {
        extract_all(&db)?;
    }

    Ok(())
}

fn get_database(path: &str, database: Option<&str>) -> Result<lmdb::Database<'static>, PresentableError> {
    let env = (unsafe {
        let mut env = lmdb::EnvBuilder::new().unwrap();
        env.set_maxdbs(2).unwrap();
        env.open(path, lmdb::open::RDONLY, 0o600)
    }).map_err(|_| PresentableError("Failed to open an environment. No database?"))?;

    lmdb::Database::open(env, database, &lmdb::DatabaseOptions::defaults())
        .map_err(|_| PresentableError("Failed to open database, does it exist?"))
}

/// Prints the keys in a database and the size of the values.
fn print_list(db: &lmdb::Database) -> Result<(), PresentableError> {
    let txn = lmdb::ReadTransaction::new(db.env())
        .map_err(|_| PresentableError("Failed to create read transaction"))?;
    let access = txn.access();
    let mut cursor = txn.cursor(db).map_err(|_| PresentableError("Failed to open a cursor"))?;
    let mut iter = lmdb::CursorIter::new(
        lmdb::MaybeOwned::Borrowed(&mut cursor),
        &access,
        |c, a| c.first(a),
        lmdb::Cursor::next::<str, [u8]>
    ).map_err(|_| PresentableError("Failed to get an iterable cursor"))?;

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

fn extract_key(db: &lmdb::Database, key: &str, out_file: &str) -> Result<(), PresentableError> {
    let txn = lmdb::ReadTransaction::new(db.env())
        .map_err(|_| PresentableError("Failed to create read transaction"))?;
    let access = txn.access();
    let data: &[u8] = access.get(&db, key)
        .map_err(|_| PresentableError("Failed to get a value by key"))?;

    let mut file = File::create(out_file)
        .map_err(|_| PresentableError("Failed to create the out file"))?;
    file.write_all(data)
        .map_err(|_| PresentableError("Failed to write to the out file"))?;

    Ok(())
}

fn extract_all(db: &lmdb::Database) -> Result<(), PresentableError> {
    let txn = lmdb::ReadTransaction::new(db.env())
        .map_err(|_| PresentableError("Failed to create read transaction"))?;
    let access = txn.access();
    let mut cursor = txn.cursor(db).map_err(|_| PresentableError("Failed to open a cursor"))?;
    let mut iter = lmdb::CursorIter::new(
        lmdb::MaybeOwned::Borrowed(&mut cursor),
        &access,
        |c, a| c.first(a),
        lmdb::Cursor::next::<str, [u8]>
    ).map_err(|_| PresentableError("Failed to get an iterable cursor"))?;

    while let Some(Ok((key, data))) = iter.next() {
        let mut file = File::create(format!("{}.bin", key))
            .map_err(|_| PresentableError("Failed to create the out file"))?;
        file.write_all(data)
            .map_err(|_| PresentableError("Failed to write to the out file"))?;
    }

    Ok(())
}
