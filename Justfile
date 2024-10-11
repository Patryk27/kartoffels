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

web:
    cd frontend \
    && npm run dev

# ---

check:
    cd backend \
    && cargo check

fmt:
    cd backend \
    && cargo fmt

test:
    cd backend \
    && cargo test

bless:
    fd -e new --no-ignore-vcs --full-path --exec mv {} {.}

# ---

bot name:
    cd backend \
    && just bot {{ name }}

bots:
    just bot dummy
    just bot roberto
