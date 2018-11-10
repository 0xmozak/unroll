# unroll

Unroll for loops with integer literal bounds. This crate provides a attribute-like macro
`unroll_for_loops` that can be applied to functions. This macro looks for loops to unroll and
unrolls them at compile time.


## Why a procedural macro?

There is already a crate called [Crunchy](https://github.com/Vurich/crunchy) that unrolls loops with
a `macro_rules!` macro. The benefits of an
item based attribute-like macro instead of a `macro_rules!` macro for each loop are:
  - Macro invocation doesn't pollute the algorithm with annotations. Only one annotation is needed
    for each function.
  - Can use variables defined outside of the macro invocation.
  - Can be extended to do partial loop unrolling


## Usage

Just add `#[unroll_for_loops]` above the function whose for loops you would like to unroll.
Currently all for loops with integer literal bounds will be unrolled, although this macro currently
can't see inside complex code (e.g. for loops within closures).


## Example

```rust
#[unroll_for_loops]
fn main() {
    println!("matrix = ");
    for i in 0..10 {
        for j in 0..10 {
            print!("({:?}, {:?})", i, j);
        }
        println!("");
    }
}
```


# Acknowledgements

I would like to thank the author of [Crunchy](https://github.com/Vurich/crunchy) for providing an
initial solution to this problem.


# License

This repository is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
