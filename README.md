# 🥔 Kartoffels 🥔

**[Kartoffels](https://kartoffels.pwy.io) is an online robot combat arena!**

Implement your own bot, submit it and see it fight other bots in real-time:

![](./readme/intro.gif)

[Play it online!](https://kartoffels.pwy.io)

## Name

kartoffel (🇩🇪)    
= potato    
= slang for "slow machine"    
= exactly what kartoffel bots are - slow machines indeed!

## Running locally

### Building on NixOS

```
$ sudo nixos-container create demo --flake '.'
$ sudo nixos-container start demo
```

... and then proceed to the `Creating a new map` section below.

### Building on other systems

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

... and then proceed below.

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

## License

Copyright (c) 2024, Patryk Wychowaniec (`pwychowaniec @at@ pm.me`).

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, version 3.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>. 
