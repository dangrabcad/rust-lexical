lexical
=======

[![Build Status](https://api.travis-ci.org/Alexhuszagh/rust-lexical.svg?branch=master)](https://travis-ci.org/Alexhuszagh/rust-lexical)
[![Latest Version](https://img.shields.io/crates/v/lexical.svg)](https://crates.io/crates/lexical)

Fast lexical conversion routines for both std and no_std environments. Lexical provides routines to convert numbers to and from decimal strings. Lexical is simple to use and focuses on performance and correctness. Finally, [lexical-core](lexical-core) is suitable for environments without a memory allocator, without any internal allocations required for the low-level API. And, as of version 2.0, lexical uses minimal unsafe features, limiting the chance of memory-unsafe code.

**Table of Contents**

- [Getting Started](#getting-started)
- [Benchmarks](#benchmarks)
- [Documentation](#documentation)
- [Roadmap](#roadmap)
- [Version Support](#version-support)
- [Changelog](#changelog)
- [License](#license)
- [Contributing](#contributing)

# Getting Started

Add lexical to your `Cargo.toml`:

```toml
[dependencies]
lexical = "^2.1"
```

And get started using lexical:

```rust
extern crate lexical;

// Number to string
lexical::to_string(3.0);            // "3.0", always has a fraction suffix, 
lexical::to_string(3);              // "3"

// String to number.
let i: i32 = lexical::parse("3");            // 3, auto-type deduction.
let f: f32 = lexical::parse("3.5");          // 3.5
let d = lexical::parse::<f64, _>("3.5");     // 3.5, explicit type hints.
let d = lexical::try_parse::<f64, _>("3.5"); // Ok(3.5), error checking parse.
let d = lexical::try_parse::<f64, _>("3a");  // Err(Error(_)), failed to parse.
```

Lexical's parsers can be either error-checked and unchecked. The unchecked parsers continue to parse until they encounter invalid data, returning a number was successfully parsed up until that point. The unchecked parsers explicitly wrap on numeric overflow. This is somewhat analogous to C's `strtod`, which may not be desirable for many applications. Therefore, lexical also includes checked parsers, which ensure the entire buffer is used while parsing, without discarding characters, and that the resulting number did not overflow. Upon erroring, the checked parsers will return the an enum indicating overflow or the index where the first invalid digit was found.

```rust
// This will return Err(Error(ErrorKind::InvalidDigit(3))), indicating 
// the first invalid character occurred at the index 3 in the input 
// string (the space character).
let x: i32 = lexical::try_parse("123 456");

// This will return Ok(123), since that is the value found before invalid
// character was encountered.
let x: i32 = lexical::parse("123 456");
```

For floating-points, Lexical also includes `parse_lossy` and `try_parse_lossy`, which may lead to minor rounding error (relative error of ~1e-16) in rare cases (see [implementation details](lexical-core/README.md#implementation-details) for more information), without using slow algorithms that lead to serious performance degradation.

```rust
let x: f32 = lexical::parse_lossy("3.5");       // 3.5
let x: f32 = lexical::try_parse_lossy("3.5");   // Ok(3.5)
```

In order to use lexical in generics, the type may use the trait bounds `FromBytes` (for `parse` and `try_parse`), `ToBytes` (for `to_string`), or `FromBytesLossy` (for `parse_lossy` and `try_parse_lossy`).

```rust
/// Multiply a value in a string by multiplier, and serialize to string.
fn mul_2<T>(value: &str, multiplier: T) 
    -> Result<String, lexical::Error>
    where T: lexical::ToBytes + lexical::FromBytes + ops::Mul<Output=T>
{
    let value: T = lexical::try_parse(value)?;
    Ok(lexical::to_string(value * multiplier))
}
```

# Benchmarks

Most of the following benchmarks measure the time it takes to convert 10,000 random values, for different types. The values were randomly generated using NumPy, and run in both std (rustc 1.29.2) and no_std (rustc 1.31.0) contexts (only std is shown) on an x86-64 Intel processor. More information on these benchmarks can be found in the [benches](benches) folder and in the source code for the respective algorithms. Adding the flags "target-cpu=native" and "link-args=-s" were also used, however, they minimally affected the relative performance difference between different lexical conversion implementations.

For cross-language benchmarks, they measure the time it takes to convert a digit series of near-halfway decimal floating-point representations. The C++ benchmarks (RapidJSON, strtod, and double-conversion) were done using GCC 8.2.1 with glibc/libstdc++ using Google Benchmark and the `-O3` flag. The Python benchmark was done using IPython on Python 3.6.6. The Go benchmark was done using go1.10.4. All benchmarks used the same data. For RapidJSON, the benchmark was done by publicly exposing the `ParseNumber` method with a custom handler.

For all the following benchmarks, lower is better.

**Float to String**

![ftoa benchmark](https://raw.githubusercontent.com/Alexhuszagh/rust-lexical/master/assets/ftoa.png)

**Integer To String**

![itoa benchmark](https://raw.githubusercontent.com/Alexhuszagh/rust-lexical/master/assets/itoa.png)

**String to Integer**

![atoi benchmark](https://raw.githubusercontent.com/Alexhuszagh/rust-lexical/master/assets/atoi.png)

**String to f64 Simple, Random Data**

![atof64 benchmark](https://raw.githubusercontent.com/Alexhuszagh/rust-lexical/master/lexical-benchmark/assets/atof_simple_f64.png)

**String to f64 Complex, Large Data Cross-Language Comparison**

![atof64 simple language benchmark](https://raw.githubusercontent.com/Alexhuszagh/rust-lexical/master/lexical-benchmark/assets/atof_large_f64.png)

**String to f64 Complex, Denormal Data Cross-Language Comparison**

Note: Rust was unable to parse all but the 10-digit benchmark, producing an error result of `ParseFloatError { kind: Invalid }`. It performed ~2,000x worse than lexical for that benchmark.

![atof64 simple language benchmark](https://raw.githubusercontent.com/Alexhuszagh/rust-lexical/master/lexical-benchmark/assets/atof_denormal_f64.png)

# Backends

For Float-To-String conversions, lexical uses one of three backends: an internal, Grisu2 algorithm, an external, Grisu3 algorithm, and an external, Ryu algorithm (~2x as fast).

# Documentation

Lexical's documentation can be found on [docs.rs](https://docs.rs/lexical).
For detailed background on the algorithms and features in lexical, see [lexical-core](lexical-core).

# Roadmap

Ideally, Lexical's float-parsing algorithm or approach would be incorporated into libcore. Although Lexical greatly improves on Rust's float-parsing algorithm, in its current state it's insufficient to be included in the standard library, including numerous "anti-features":

1. It supports non-decimal radices for float parsing, leading to significant binary bloat and increased code branching, for almost non-existent use-cases.
2. It supports rounding schemes other than round-to-nearest, tie-even.
3. It inlines aggressively, producing significant binary bloat.
4. It contains effectively dead code for efficient higher-order arbitrary-precision integer algorithms, for rare use-cases requiring asymptotically faster algorithms.

# Version Support

Lexical is tested to work from Rustc versions of 1.24-1.35, and should work on newer versions as well. Please report any errors compiling lexical for any Rust compiler 1.24.0 or later.

# Changelog

All changes since 2.2.0 are documented in [CHANGELOG](CHANGELOG).

# License

Lexical is dual licensed under the Apache 2.0 license as well as the MIT license. See the LICENCE-MIT and the LICENCE-APACHE files for the licenses. 

# Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in lexical by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
