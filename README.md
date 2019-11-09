# nonstdio
Bart Massey

This crate is a WIP technology demo for a new design for
Rust stdio.

## Background

`Stdout` / `StdoutLock` always wraps an internal
`LineWriter` that scans the output for line breaks and
flushes there; this in turn sits atop a 1024-byte
(hardcoded) `BufWriter`. If you do a larger write to
`Stdout` the `LineWriter` will flush the current buffer and
pass the write through. As far as I can tell there is no way
to stop the newline scanning.

The underlying raw file descriptor (UNIX) or file handle
(Windows) can be extracted; for UNIX at least it is probably
harmless to just use file descriptor 1 rather than `AsRawFd`
— it would be strange for a UNIX system to use anything
other fd. As far as I can tell, there's no portable way to
get the underlying raw output file as a rust object that
implements `Write` — I'm using `FromRawFd::from_raw_fd()` to
get a `File`, which is `unsafe` and UNIX-specific. There is
a `StdoutRaw` struct in `std`, but it's private and not
exposed.

Deficiencies with current stdio as I see them (I'm probably
wrong as usual, but this is my take):

* stdin implements only `Read`, and stdout and stderr
  implement only `Write`. This is normally OK, but there are
  in particular times when the ability to read stderr
  (e.g. for passwords) would be a good idea.

* A minor annoyance: all the stdio structs have unique
  types. This makes it harder to write things that
  manipulate stdio in a generic way. For example, opening
  `/dev/tty` on UNIX gives a `File` that it would be nice to
  be able to handle by whatever paths `Stdout` and `Stdin`
  are using. As another example, the `atty` crate wraps the
  stdio structs in its own `Stream` struct to get uniform
  behavior for them.

* There seems to be no way to suppress the newline scanning
  on stdout. This is a quite notable performance hit in some
  scenarios (I've measured).

* There seems to be no way to directly increase the
  underlying buffer size for stdout. The workaround I've
  found is to wrap stdout in a larger `BufWriter` and let
  the pass-through do its thing. This is also a notable
  performance hit: output to stdout is now going through
  three layers of indirection (`BufWriter` → `LineWriter` →
  `BufWriter`) before hitting a syscall. The inliner may or
  may not save you here; in my measurements it does not seem
  to.

* There seems to be no way to suppress the buffering on
  stdout.  This is a quite notable performance hit in some
  scenarios (I've measured) compared to doing the buffering
  directly in app code in an app-specific fashion.

* The API around locking stdio is confusing to new users and
  annoying to me. I hesitate to bring this up, since I don't
  have a better design in mind. Because Rust crates can
  freely spin up their own threads that use stdio, the
  locking seems unavoidable.

  It seems like `StdoutLocked` should be able to hold a
  reference to its underlying `RefCell<Stdout>` so that the
  caller doesn't have to, but that would require that
  `stdout()` clones the `RefCell` I guess?
  
* The API in general is awkward and error-prone. See
    https://play.rust-lang.org/?gist=4f56375c11978a75bb18e480250e04f8

This crate is how I think (?) I wish Rust stdio worked. This sketch
is inspired pretty heavily by UNIX stdio as it has evolved.
It probably has a number of problems.
