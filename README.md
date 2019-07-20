# lmdb-helper

```text
bibliofile ~/Desktop $ lmdb-helper --help
lmdb-helper 0.1
Bibliofile <bibliofilegit@gmail.com>
Simple tool to extract information about LMDB databases

USAGE:
    lmdb-helper [FLAGS] [OPTIONS] [DIR]

FLAGS:
    -h, --help       Prints help information
    -l, --list       If passed, only print the possible keys in the database, defaults to true if --extract is not
                     passed
    -V, --version    Prints version information

OPTIONS:
    -d, --database <DATABASE>    The database to read, if no database is passed and the lmdb uses named databases, the
                                 returned keys will be the possible database names
    -e, --extract <extract>      If passed, extract the value in the database to a file as specified by --out
    -o, --out <out>              Specify the name of the extracted file, defaults to <key>.bin if not specified.

ARGS:
    <DIR>    Sets the database directory to use, defaults to the current working directory.
 ```
