# (s)imple (ex)pression (e)valuator
[![Build Status](https://travis-ci.org/z2oh/sexe.svg?branch=master)](https://travis-ci.org/z2oh/sexe)

<center>
    <img src="demo.png">
</center>

There are three components to this project:

- A system to model and evaluate arbitrary arithmetic expressions ([src/expression.rs](src/expression.rs))
- A parser to parse human written expressions into an expression model that can be evaluated ([src/parser.rs](src/parser.rs))
- A graphing interface for the terminal, to graph functions across some domain ([src/interface.rs](src/interface.rs))

Currently, the operators are implemented:
* Addition
* Subtraction
* Multiplication
* Division
* Sin
* Cos
* Tan
* Abs
* Log2
* Ln
* Negation

Want to add a new function/operator? Make an issue or PR!

## Installation

Rust and Cargo must be installed. I recommend using [rustup](https://rustup.rs/).

```
git clone https://github.com/z2oh/sexe
cd sexe
cargo build --release
```

The binary is then located at `./target/release/sexe`.
