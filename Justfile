run *args:
    cd backend \
    && cargo run --release -- \
           serve \
           ../data \
           --http 127.0.0.1:1313 \
           --ssh 127.0.0.1:1314 \
           {{ args }}

toolbox *args:
    cd backend \
    && cargo run --release -- \
           toolbox \
           {{ args }}

bot name:
    cd backend \
    && cargo build \
           -p bot-{{ name }} \
           --release \
           --target riscv64-kartoffel-bot.json \
           -Z build-std \
           -Z build-std-features=compiler-builtins-mem

bots:
    just bot dummy
    just bot roberto

web:
    cd frontend \
    && npm run dev

test:
    cd backend \
    && cargo test
