# lmdb-helper

```text
$ lmdb-helper --help
lmdb-helper 0.2
Bibliofile <bibliofilegit@gmail.com>
Simple tool to view & modify LMDB databases

USAGE:
    lmdb-helper [FLAGS] [OPTIONS] [DIR]

FLAGS:
        --extract-all    If passed, extract all keys from a database as <key>
    -h, --help           Prints help information
    -l, --list           If passed, only print the possible keys in the database, defaults to true if neither --extract
                         nor --insert is passed
    -V, --version        Prints version information

OPTIONS:
    -d, --database <DATABASE>    The database to read, if no database is passed and named databases are used, the
                                 returned keys will be the possible database names
    -e, --extract <extract>      If passed, extract the value in the database to a file as specified by --out
    -i, --insert <insert>        If passed, inserts the given file into the database, using the name as the key
    -o, --out <out>              Specify the name of the extracted file, defaults to <key> if not specified.

ARGS:
    <DIR>    Sets the database directory to use, defaults to the current working directory.
 ```

## Compiling

If you have rust installed, just running `cargo build` will create an executable you can use. If you need a 32 bit version you'll need to compile with something like `cargo build --target=i686-pc-windows-msvc`
