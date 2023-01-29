# serfig &emsp; [![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://img.shields.io/github/actions/workflow/status/Xuanwo/serfig/ci.yml?branch=main
[actions]: https://github.com/Xuanwo/serfig/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/serfig.svg
[crates.io]: https://crates.io/crates/serfig

Layered configuration system built upon serde

## Quick Start

```rust
use serde::{Deserialize, Serialize};
use serfig::collectors::{from_env, from_file, from_self};
use serfig::parsers::Toml;
use serfig::Builder;

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
struct TestConfig {
    a: String,
    b: String,
    c: i64,
}

fn main() -> anyhow::Result<()> {
    let builder = Builder::default()
        .collect(from_env())
        .collect(from_file(Toml, "config.toml"))
        .collect(from_self(TestConfig::default()));
    let t: TestConfig = builder.build()?;

    println!("{:?}", t);
    Ok(())
}
```

## Contributing

Check out the [CONTRIBUTING.md](./CONTRIBUTING.md) guide for more details on getting started with contributing to this project.

## Getting help

Submit [issues](https://github.com/Xuanwo/serfig/issues/new/choose) for bug report or asking questions in [discussion](https://github.com/Xuanwo/serfig/discussions/new?category=q-a).

## Acknowledgment

This project is highly inspired by [config-rs](https://github.com/mehcode/config-rs)

#### License

<sup>
Licensed under <a href="./LICENSE">Apache License, Version 2.0</a>.
</sup>
