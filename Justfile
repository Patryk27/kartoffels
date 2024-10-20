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
    && cargo check --workspace \
    && cargo check --workspace --tests \
    && cargo clippy --workspace \
    && cargo clippy --workspace --tests

fmt:
    cd backend \
    && cargo fmt

test:
    cd backend \
    && cargo test --release --workspace

bless:
    fd -e new --no-ignore-vcs --full-path --exec mv {} {.}

bots:
    cd backend \
    && just bots

# ---

start-demo:
    sudo nixos-container update demo --flake .
    sudo nixos-container start demo
    @echo
    @echo "ready:"
    @echo "http://$(nixos-container show-ip demo)"

stop-demo:
    sudo nixos-container stop demo
