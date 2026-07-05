# bust — B compiler for x86-64 Linux

bust is a compiler for [B](<https://en.wikipedia.org/wiki/B_(programming_language)>), the language that preceded C at Bell Labs. It targets x86-64 Linux and produces ELF binaries.

## Usage

```
bust source.b -o output         # compile & link → executable
bust -S source.b -o output.s    # assembly only
bust -s 'printn(42);'           # one-liner
```

### Options

| Option | Description |
|---|---|
| `-o <file>` | Output file (default: `a.out`) |
| `-S` | Emit assembly only, do not assemble/link |
| `-s <code>` | Compile string instead of file |
| `--libb-path <path>` | Path to `liblibb.a` |

## Install

```
cargo xtask install                      # → /usr/local/{bin,lib64}
cargo xtask install --prefix ~/.local    # → ~/.local/{bin,lib64}
```

```
cargo xtask uninstall                    # remove from /usr/local
cargo xtask uninstall --prefix ~/.local  # remove from ~/.local
```

`liblibb.a` is resolved at runtime in this order: `--libb-path` → `LIBB_PATH` env → well-known system paths → compile-time fallback.

## Project structure

| Path | Description |
|---|---|
| `bust/` | Compiler: lexer, parser, codegen |
| `libb/` | Runtime: syscall wrappers, I/O, no libc |
| `xtask/` | Build & install automation |

## Example projects

Larger programs written in B using `bust` can be found here:

- [Bad Bapple](https://github.com/sonidori98/Bad-Bapple/tree/main) — Bad Apple!! rendered as ASCII art in B.

## Test

```
cargo xtask test
```

## References

- [BCause](https://github.com/Spydr06/BCause) — C implementation that inspired this project (inspiration)
- [blang](https://github.com/sergev/blang) — Go/LLVM implementation (inspiration)
- [Users' Reference to B](https://raw.githubusercontent.com/sergev/blang/refs/heads/main/doc/kbman.pdf) by Ken Thompson (1972)

## License

MIT
