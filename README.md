# Cosmwasm Starter Pack

This is a template to build smart contracts in Rust to run inside a 
[Cosmos SDK](https://github.com/cosmos/cosmos-sdk) module on all chains that enable it.
To understand the framework better, please read the overview in the
[cosmwasm repo](https://github.com/confio/cosmwasm/blob/master/README.md).
This assumes you understand the theory and just want to get coding.

## Creating a new repo from template

Before starting, make sure you have [rustup](https://rustup.rs/) along with a recent `rustc` and `cargo` 
version installed. Currently, we are testing on 1.37+. 
 
And you need to have the `wasm32-unknown-unknown` target installed as well.

You can check that via:

```shell script
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

You will also need to have 
[cargo generate](https://github.com/ashleygwilliams/cargo-generate) installed.
Unless you did that before, run this line now:

```shell script
cargo install cargo-generate --features vendored-openssl
```

Now that the prerequisites are over, you can create your new contract.
Go to the folder in which you want to place it and run:

```shell script
cargo generate --git https://github.com/confio/cosmwasm-template.git --name YOUR_NAME_HERE
```

You will now have a new folder called `YOUR_NAME_HERE` (I hope you changed that to something else)
containing a simple working contract and build system that you can customize.

## Compiling and running tests

Now that you created your custom contract, make sure you can compile and run it before
making any changes. Go into the 

```shell script
# this will produce a wasm build in ./target/wasm32-unknown-unknown/release/YOUR_NAME_HERE.wasm
cargo wasm

# this runs unit tests with helpful backtraces
RUST_BACKTRACES=1 cargo unit-test
RUST_BACKTRACES=1 cargo test -lib --features backtraces

# this runs integration tests with cranelift backend (uses rust stable)
cargo test

# this runs integration tests with singlepass backend (needs rust nightly)
cargo test --no-default-features --features singlepass
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
if you do `RUST_BACKTRACE=1 cargo test`.

However, we don't just want to test the logic rust, but also the compiled wasm artifact 
inside a vm. You can look in `tests/integration.rs` to see some examples there. They
load the wasm binary into the vm and call the contract externally. Effort has been
made that the syntax is very similar to the calls in the native rust contract and
quite easy to code. In fact, usually you can just copy a few unit tests and modify
a few lines to make an integration test (this should get even easier in a future release).

We consider testing critical for anything on a blockchain, and recommend to always keep
the tests up to date. While doing active development, it is often simplest to disable
the integration tests completely and iterate rapidly on the code in `contract.rs`,
both the logic and the tests. Once the code is finalized, you can copy over some unit
tests into the integration.rs and make the needed changes. This ensures the compiled
wasm also behaves as desired in the real system.

## Preparing the wasm for production

Before we upload it to a chain, we need to ensure the smallest output size possible,
as this will be included in the body of a transaction. We also want to have a
reproducible build process, so third parties can verify that the uploaded wasm
code did indeed come from the claimed rust code.

To solve both these issues, we have produced `cosmwasm-opt`, a docker image to
produce an extremely small build output in a consistent manner. To use it,

Linux: `docker run --rm -u $(id -u):$(id -g) -v $(pwd):/code confio/cosmwasm-opt:0.4.1`

This produces a `contract.wasm` file in the current directory (which must be the root
directory of your rust project, the one with `Cargo.toml` inside). The current sample
contract compiles down to around 48kB wasm file.

Note this will take a while, as it doesn't share the cargo registry nor the incremental
compilation cache with your host system, in order to provide the most consistent setup.

We also track the versions of cosmwasm that we aim for compatibility. The most important
aspect is the same version of wasm-pack and wasm-bindgen. For 0.4.1 we are tied to
wasm-pack 0.8.1, wasm-bindgen 0.2.53, and rust 1.38.