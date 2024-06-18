# emptydir

This tool looks for empty directories and deletes them.

```console
$ emptydir 
```

More specifically, it deletes directories which are completely empty, or which only contain files/folders which I don't think are worth keeping (e.g. `.DS_Store`).





## Installation

You can download compiled binaries from the [GitHub releases](https://github.com/alexwlchan/emptydir/releases).

Alternatively, you can install from source.
You need Rust installed; I recommend using [Rustup].
Then clone this repository and compile the code:

```console
$ git clone "https://github.com/alexwlchan/emptydir.git"
$ cd emptydir
$ cargo install --path .
```

[Rustup]: https://rustup.rs/





## Usage

Pass the path of the top-level directory you want to clean as a command-line argument, for example:

```console
$ emptydir ~/Desktop
```

It will print the path to every empty directory that it deletes:

```console
$ mkdir -p ~/Desktop/foo/bar/baz
$ emptydir ~/Desktop/
/Users/alexwlchan/Desktop/foo/bar/baz
/Users/alexwlchan/Desktop/foo/bar
/Users/alexwlchan/Desktop/foo
3 directories deleted
```

If you pass no arguments, it will look for empty directories in the current directory:

```console
$ emptydir
```



## License

MIT.
