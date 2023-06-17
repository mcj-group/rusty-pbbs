***Disclaimer:*** This repo contains 4 of the 12 benchmarks listed in our
paper. The rest will be coming shortly. And here is the paper:



[Brief Announcement: Is the Problem-Based Benchmark Suite Fearless with Rust?](https://doi.org/10.1145/3558481.3591313)<br>
Javad Abdi, Guowei Zhang, Mark C. Jeffrey<br>
ACM Symposium on Parallelism in Algorithms and Architectures (SPAA), 2023


# Rusty-PBBS
A replica of PBBS in Rust.

# Build

## Install Rust (cargo, rustc, ...)

```bash
# download and install the version management tool: Rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# confiqure the current shell
source "$HOME/.cargo/env"
```

The `default` features are enough for benchmarking.
If you want to do more hacking, the `complete` porfile could help.

## Download rusty-pbbs

```bash
git clone https://github.com/mcj-group/rusty-pbbs.git
cd rusty-pbbs/
```

## Configure the build system (optional)

### Create a config file

```bash
mkdir .cargo
touch .cargo/config.toml
```

### Setup the config file
Open the created file and use the following configuration.
Most of these make small performance changes or decrease compile time.

```toml
[build]
target-dir = "/path/to/build/directory/pbbs"

[target.x86_64-unknown-linux-gnu] # replace with your target
linker = "/path/to/linker"
rustflags = [
    "-C", "target-cpu=native", # change to other CPUs when cross compiling
    #"-C", "link-arg=--ld-path=/path/to/linker",
]

[profile.release]
opt-level       = 3      # Optimization level
debug           = 0      # Include debug info
debug-assertions = false # Enables debug assertions
codegen-units   = 1      # Number of code generation units
# lto            = true    # link-time optimization
# panic          = "abort" # abort upon panics
overflow-checks = false
```


### install gcc if you don't have it on your system.

## Compile benchmarks

```bash
cargo build --release                # compile all benchmarks
cargo build --release --bin="dedup"  # compile an specific benchmark (dedup)
```

# Run
Cargo can run an individual benchmark (e.g. dedup):
```bash
cargo run --release --bin=dedup -- <input_file>
```
or the binary can be run itself:
```bash
/path/to/build/directory/pbbs/release/dedup <input_file>
```

To get the full list of flags and arguments use `--help`:
```bash
/path/to/build/directory/pbbs/release/dedup --help
```

## Example

Let's, run parallel dedup for 10 rounds on an input.
```bash
$ /.../dedup -o outfile -a parhash -r 3 /path/to/input

dedup:	2.560179
dedup:	2.608721
dedup:	2.492258
OutLoopTime:total:	12.966530
2.578050534s
```

# Acknowledgements

This project was inspired by the algorithms from the following sources:

- The problem-based benchmark suite: [github repo](https://github.com/cmuparlay/pbbsbench) and a [relevant paper](https://dl.acm.org/doi/10.1145/3503221.3508422) (License: [MIT](https://github.com/cmuparlay/pbbsbench/blob/master/LICENSE))
- ParlayLib: [github repo](https://github.com/cmuparlay/parlaylib) (License: [MIT](https://github.com/cmuparlay/parlaylib/blob/master/LICENSE))
