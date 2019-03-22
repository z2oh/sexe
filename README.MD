# (s)imple (ex)pression (e)valuator
[![Build Status](https://travis-ci.org/z2oh/sexe.svg?branch=master)](https://travis-ci.org/z2oh/sexe)

<center>
    <img src="demo.png" alt="Thanks to Michael Gottlieb for this neat looking function! mzrg.com/">
</center>

There are three components in this project:

- An expression engine that models and evaluates arbitrary arithmetic expressions ([sexe-expression](sexe-expression)).
- A parser to convert human written expressions into a model that can be evaluated with the expression engine ([sexe-parser](sexe-parser)).
- A graphing interface for the terminal, to graph functions across a specified domain ([src/interface.rs](src/interface.rs)).

There is a variety of supported syntax when writing functions. 

Is something missing? Make an issue or PR!

## Installation

Rust and Cargo must be installed. I recommend using [rustup](https://rustup.rs/).

```
git clone https://github.com/z2oh/sexe
cd sexe
cargo build --release
```

The binary is then located at `./target/release/sexe`.
