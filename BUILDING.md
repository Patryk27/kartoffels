## Building

### Building on NixOS

```
$ sudo nixos-container create demo --flake '.'
$ sudo nixos-container start demo
```

... and then proceed to the `Creating a new map` section below.

### Building on other systems

Build roberto, an example bot required for UI:

```
$ cd backend
$ cargo build -p roberto --release --target misc/riscv64-kartoffel-bot.json -Z build-std -Z build-std-features=compiler-builtins-mem
```

Build the sandbox:

```
$ cd backend
$ wasm-pack build ./crates/kartoffels-sandbox --target web
```

Build the backend and start it:

```
$ mkdir /tmp/kartoffels
$ cd backend
$ cargo run --release -- --data /tmp/kartoffels
```

Build the frontend and start it:

```
$ cd frontend
$ npm install
$ npm run dev
```

... and then proceed to the section below.

### Creating a new map

By default, the server starts empty - you can create a map by issuing a request:

```
POST http://localhost:1313/worlds
Content-Type: application/json

{
  "name": "total mayhem",
  "mode": {
    "type": "deathmatch"
  },
  "theme": {
    "type": "arena",
    "radius": 40
  },
  "policy": {
    "max_alive_bots": 128,
    "max_queued_bots": 256
  }
}
```

(if you're using NixOS containers, the endpoint will most likely be
`POST http://10.233.1.2/api/worlds`)

Having done that, enjoy!
