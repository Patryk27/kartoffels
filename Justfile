back:
    cd backend \
    && cargo run --release -p kartoffels-server -- --data ../data

bot name:
    cd backend \
    && cargo build \
        -p bot-{{ name }} \
        --release \
        --target misc/riscv64-kartoffel-bot.json \
        -Z build-std \
        -Z build-std-features=compiler-builtins-mem

bots:
    just bot dummy
    just bot roberto

wasm:
    cd backend \
    && nix develop \
        -c wasm-pack build ./crates/kartoffels-sandbox --target web

front:
    cd frontend \
    && npm run dev

fmt:
    cd backend \
    && cargo fmt

    cd frontend \
    && prettier . --write

check:
    cd backend \
    && cargo check

    cd frontend \
    && npm exec vue-tsc
