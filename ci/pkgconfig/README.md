# CI pkg-config shims (typecheck only)

These `.pc` files let `cargo check`/`clippy` typecheck the Tauri shell in an
environment that has Rust and Node but **not** the WebKitGTK development
packages. They only satisfy dependency discovery; they do not provide headers or
libraries and are never used to link or run the application.

`scripts/ci.sh` uses this directory only when the real packages are absent. On a
machine with `webkit2gtk-4.1`, `javascriptcoregtk-4.1` and `libsoup-3.0` from
pkg-config, the CI uses the real environment and the native link is validated by
the normal build lane.

Do not use these shims to produce a release binary.
