# API Rest with Rust and Actix Web
This project aims to present a Rust project with Actix Web

Technologies used: Rust, Actix Web, Chrono, Serde, Async-Trait, Tracing, Sqlx, Postgres, Mockall, httpMock

### Pre-Requires
  - rust and cargo ([Install](https://www.rust-lang.org/tools/install))
  - cargo-make ([Install](https://github.com/sagiegurari/cargo-make))
  - podman ([Install](https://podman.io/getting-started/installation))
  - sqlx-cli ([Install](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli))

### Podman Commands

  - Run Postgres on Container

  `podman run -d --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=mi_api postgres`

  - Stop container

  `podman stop postgres`

  - Start container

  `podman start postgres`


### API Commands

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






