apt install nasm gcc libgtk-3-dev pkg-config curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
cargo install --path .
