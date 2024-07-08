backend:
    cd backend && cargo run --release -p kartoffels-server -- --data ../data

backend-wasm:
    cd backend && nix develop -c wasm-pack build ./crates/kartoffels-sandbox --target web

frontend:
    cd frontend && nix develop -c npm run dev

tsc:
    cd frontend && nix develop -c npm exec vue-tsc

roberto:
    cd backend && cargo build -p roberto --release --target misc/riscv64-kartoffel-bot.json -Z build-std -Z build-std-features=compiler-builtins-mem
