back:
    cd backend && cargo run --release -p kartoffels-server -- --data ../data

wasm:
    cd backend && nix develop -c wasm-pack build ./crates/kartoffels-sandbox --target web

front:
    cd frontend && nix develop -c npm run dev

fmt:
    cd backend && cargo fmt
    cd frontend && prettier . --write

check:
    cd backend && cargo check
    cd frontend && nix develop -c npm exec vue-tsc

roberto:
    cd backend && cargo build -p roberto --release --target misc/riscv64-kartoffel-bot.json -Z build-std -Z build-std-features=compiler-builtins-mem
