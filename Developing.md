# Developing

If you have recently created a contract with this template, you probably could use some
help on how to build and test the contract, as well as prepare it for production. This
file attempts to provide a brief overview, assuming you have installed a recent
version of rust already (eg. 1.38+).

## Prerequisites

Before starting, make sure you have [rustup](https://rustup.rs/) along with a recent `rustc` and `cargo`
version installed. Currently, we are testing on 1.38+.

And you need to have the `wasm32-unknown-unknown` target installed as well.

You can check that via:

```sh
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

## Compiling and running tests

Now that you created your custom contract, make sure you can compile and run it before
making any changes. Go into the

```sh
# this will produce a wasm build in ./target/wasm32-unknown-unknown/release/YOUR_NAME_HERE.wasm
cargo wasm

# this runs unit tests with helpful backtraces
RUST_BACKTRACE=1 cargo unit-test

# this runs integration tests with cranelift backend (uses rust stable)
cargo integration-test

# this runs integration tests with singlepass backend (needs rust nightly)
cargo integration-test --no-default-features --features singlepass

# auto-generate json schema
cargo schema
```

The wasmer engine, embedded in `cosmwasm-vm` supports multiple backends:
singlepass and cranelift. Singlepass has fast compile times and slower run times,
and supportes gas metering. It also requires rust `nightly`. This is used as default
when embedding `cosmwasm-vm` in `go-cosmwasm` and is needed to use if you want to
check the gas usage.

However, when just building contacts, if you don't want to worry about installing
two rust toolchains, you can run all tests with cranelift. The integration tests
may take a small bit longer, but the results will be the same. The only difference
is that you can not check gas usage here, so if you wish to optimize gas, you must
switch to nightly and run with cranelift.

### Understanding the tests

The main code is in `src/contract.rs` and the unit tests there run in pure rust,
which makes them very quick to execute and give nice output on failures, especially
if you do `RUST_BACKTRACE=1 cargo unit-test`.

However, we don't just want to test the logic rust, but also the compiled Wasm artifact
inside a VM. You can look in `tests/integration.rs` to see some examples there. They
load the Wasm binary into the vm and call the contract externally. Effort has been
made that the syntax is very similar to the calls in the native rust contract and
quite easy to code. In fact, usually you can just copy a few unit tests and modify
a few lines to make an integration test (this should get even easier in a future release).

To run the latest integration tests, you need to explicitely rebuild the Wasm file with
`cargo wasm` and then run `cargo integration-tests`.

We consider testing critical for anything on a blockchain, and recommend to always keep
the tests up to date. While doing active development, it is often simplest to disable
the integration tests completely and iterate rapidly on the code in `contract.rs`,
both the logic and the tests. Once the code is finalized, you can copy over some unit
tests into the integration.rs and make the needed changes. This ensures the compiled
Wasm also behaves as desired in the real system.

## Generating JSON Schema

While the Wasm calls (`init`, `handle`, `query`) accept JSON, this is not enough
information to use it. We need to expose the schema for the expected messages to the
clients. You can generate this schema by calling `cargo schema`, which will output
4 files in `./schema`, corresponding to the 3 message types the contract accepts,
as well as the internal `State`.

These files are in standard json-schema format, which should be usable by various
client side tools, either to auto-generate codecs, or just to validate incoming
json wrt. the defined schema.

## Preparing the Wasm bytecode for production

Before we upload it to a chain, we need to ensure the smallest output size possible,
as this will be included in the body of a transaction. We also want to have a
reproducible build process, so third parties can verify that the uploaded Wasm
code did indeed come from the claimed rust code.

To solve both these issues, we have produced `cosmwasm-opt`, a docker image to
produce an extremely small build output in a consistent manner. To use it,

Linux: `docker run --rm -u $(id -u):$(id -g) -v $(pwd):/code confio/cosmwasm-opt:0.4.1`

This produces a `contract.wasm` file in the current directory (which must be the root
directory of your rust project, the one with `Cargo.toml` inside). The current sample
contract compiles down to around 60kB Wasm file.

Note this will take a while, as it doesn't share the cargo registry nor the incremental
compilation cache with your host system, in order to provide the most consistent setup.

We also track the versions of cosmwasm that we aim for compatibility. The most important
aspect is the same version of wasm-pack and wasm-bindgen. For 0.4.1 we are tied to
wasm-pack 0.8.1, wasm-bindgen 0.2.53, and rust 1.38.