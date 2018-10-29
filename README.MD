# (s)imple (ex)pression (e)valuator

[![Build Status](https://travis-ci.org/z2oh/sexe.svg?branch=master)](https://travis-ci.org/z2oh/sexe)

<center>
    <img src="demo.png" alt="Thanks to Michael Gottlieb for this neat looking function! mzrg.com/">
</center>

There are three components to this project:

- A system to model and evaluate arbitrary arithmetic expressions ([src/expression.rs](src/expression.rs))
- A parser to parse human written expressions into an expression model that can be evaluated ([src/parser.rs](src/parser.rs))
- A graphing interface for the terminal, to graph functions across some domain ([src/interface.rs](src/interface.rs))

There is a variety of supported syntax when writing functions. Is something missing? Make an issue or PR!

## Installation

Rust and Cargo must be installed. I recommend using [rustup](https://rustup.rs/).

```
git clone https://github.com/z2oh/sexe
cd sexe
cargo build --release
```

The binary is then located at `./target/release/sexe`.

## Benchmarking statistics

Running the command `rustup run nightly cargo bench` will result in some generic benchmarks for function evaluation.

Below are some results for certain expressions:

```
test interface::tests::complicated_expression_100res    ... bench:      67,226 ns/iter (+/- 19,984)
test interface::tests::complicated_expression_amillires ... bench: 327,308,599 ns/iter (+/- 4,189,208)
test interface::tests::sine_expression_100res           ... bench:      11,714 ns/iter (+/- 1,650)
test interface::tests::sine_expression_amillires        ... bench:  80,355,897 ns/iter (+/- 8,516,084)
test interface::tests::single_variable_100res           ... bench:       8,075 ns/iter (+/- 1,122)
test interface::tests::single_variable_amillires        ... bench:  62,609,645 ns/iter (+/- 4,351,933)
```

Where the complicated_expression is `|sin(x^x) / 2^((x^x - pi/2)/pi)|`

As you can see, this complicated expression evaluated at a resolution of 1,000,000 only took 0.327 seconds to evaluate. That's pretty fast!
