# Logico

`logico` is a small CLI to evaluate logical expressions.

## Build

`logico` is written in Rust, type `cargo build` or `cargo build --release` to build it.
## Run

Examples:

Evaluate `a and b`:
```
# logico 'a & b'
| a | b ||   |
+---+---++---+
| 0 | 0 || 0 |
| 1 | 0 || 0 |
| 0 | 1 || 0 |
| 1 | 1 || 1 |
```

Evaluate `a&b=a|b|c` with `c` preset to `true`:
```
# logico 'a&b=a|b|c' +c
| a | b | c ||   |
+---+---+---++---+
| 0 | 0 | 1 || 0 |
| 1 | 0 | 1 || 0 |
| 0 | 1 | 1 || 0 |
| 1 | 1 | 1 || 1 |
```

Type `logico` without any parameters to get help.

