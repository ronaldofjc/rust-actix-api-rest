# API Rest with Rust and Actix Web
This project aims to present a Rust project with Actix Web

Technologies used: Rust, Actix Web, Chrono, Serde, Async-Trait, Tracing, Sqlx, Postgres

### Pre-Requires
  - Rust and Cargo ([Install](https://www.rust-lang.org/tools/install))
  - cargo-make
  `cargo install --force cargo-make`
  - podman ([Install](https://podman.io/getting-started/installation))
  - sqlx-cli ([Link](https://github.com/launchbadge/sqlx) | [Install](cargo install sqlx-cli))

### Commands

  - Compile project on develop

   `cargo build`

  - Compile on release

  `cargo build --release`

  - Execute on develop without cargo-make

  `cargo watch -x run`

  - Execute on develop with cargo-make

  `makers dev`

  - Execute on release without cargo-make

  `cargo run --release`

  - Execute on release with cargo-make

  `makers dev-r`






