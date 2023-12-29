work:
    cargo watch
build:
    cargo build --release
work-wasm:
    (cd dist && python -m http.server) &
    trunk watch --release --features comfy/ci-release
build-wasm:
    trunk build --release --features comfy/ci-release
prepare-tools:
    cargo install --locked trunk
    cargo install --locked cargo-watch
