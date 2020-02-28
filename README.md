# gitstatus

This is a Rust re-implementation of `gitstatus.py`, part of
[zsh-git-prompt], taking some inspiration for the alternative Haskell
implementation included with [zsh-git-prompt]. It can be used as a
drop-in replacement for the Python script.

The current differences to `gitstatus.py` are:

- Implemented with the [Rust libgit2 bindings], so the binary is
  self-contained and does not fork git, which should be beneficial for
  performance.
- Handles unborn branches, so you get a status for a
  freshly-initialized repository. For an unborn branch, a question
  mark (`?`) is shown as a branch name.

[zsh-git-prompt]: https://github.com/olivierverdier/zsh-git-prompt
[libgit2 bindings]: https://crates.io/crates/git2

## Prompt structure

Please refer to the [zsh-git-prompt's README] for information on the
status information shown.

[zsh-git-prompt's README]: https://github.com/olivierverdier/zsh-git-prompt/blob/master/README.md

## Installation

1. You need a Rust toolchain to build `gitstat`; the easiest way
   obtain a recent one is via [rustup]. This will provide you with
   `cargo`, Rust's package manager, which you'll invoke below to build
   and/or install `gitstat`.

2. Once `gitstat` is published to [crates.io], you will be able to
   install or update it to its latest release with:

    ```sh
    cargo install --force gitstat
    ```

    In the meantime, use `cargo build --release` in the top-level
    directory of the source code, and copy the executable placed in
    `target/release/gitstat` into your `$PATH`.

3. As with [zsh-git-prompt], source the file `zshrc.sh` from your
    `~/.zshrc` config file, and configure your prompt. So, somewhere
    in `~/.zshrc`, you should have:

    ```sh
    source path/to/zshrc.sh
    # an example prompt
    PROMPT='%B%m%~%b$(git_super_status) %# '
    ```

    Instead of allowing choosing between different implementations, as
    [zsh-git-prompt] does, you can set `GITSTAT_COMMAND`, which should
    work also when referring to `gitstatus.py`, but then you'd lose
    all the performance goodness.

[rustup]: https://rustup.rs/
[crates.io]: https://crates.io/

### Static build

For deployment to a Linux target, an attractive option is to create a
statically linked binary using Rust's MUSL target. This will result in
a completely standalone binary, which depends only on the Linux
kernel's system call ABI.

```sh
# If you haven't installed the MUSL target already, let's do that now:
rustup target add x86_64-unknown-linux-musl
# Build against the MUSL libc target
cargo build --target x86_64-unknown-linux-musl --release
# Let's check it's really a static binary
file target/x86_64-unknown-linux-musl/release/tdns \
  | grep -q 'statically linked' || echo "nope"
```

## License

The code and documentation of `gitstat` is [free
software](https://www.gnu.org/philosophy/free-sw.html), licensed under
the [MIT license]. It includes a variant of the `zshrc.sh` script from
[zsh-git-prompt], which is also provided under the [MIT license].

[MIT license]: ./LICENSE
