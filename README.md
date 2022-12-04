# Client lib for OSS Vizier

[![Rust](https://github.com/ssoudan/oss-vizier/actions/workflows/rust.yml/badge.svg)](https://github.com/ssoudan/oss-vizier/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/oss-vizier)](https://crates.io/crates/oss-vizier)
[![Documentation](https://docs.rs/oss-vizier/badge.svg)](https://docs.rs/oss-vizier)
[![Crates.io](https://img.shields.io/crates/l/oss-vizier)](LICENSE)

Unofficial client library for the [OSS Vizier](https://github.com/google/vizier)
service.

# License

Licensed under Apache-2.0. See [LICENSE](./LICENSE) for details.

# Examples

```bash
conda env create -f environment.yml
conda activate oss-vizier

python run_server.py & 

cargo run --example e2e
```



See [`examples`].

