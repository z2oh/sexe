# (s)imple (ex)pression (e)valuator

There are three components to this project:
* A system to model and evaluate arbitrary arithmetic expressions
* A parser to parse human written expressions into an expression model that can be evaluated
* A graphing interface for the terminal, to graph functions across some domain

Currently, only the arithmetic expression modeling is implemented with a limited set of operators (addition, subtraction, multiplication, division, negation, sin, and cosin).

## Installation
Rust and Cargo must be installed. I recommend using [rustup](https://rustup.rs/).

```
git clone https://github.com/z2oh/sexe
cd split_word
cargo build --release
```

The binary is then located at `./target/release/sexe`.
