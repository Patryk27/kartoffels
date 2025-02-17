# kartoffels 🥔

<p align="center">
    <a href="https://kartoffels.pwy.io">kartoffels</a> is a game where you're
    given a potato and your job is to implement
    <a href="https://github.com/patryk27/kartoffel">a firmware</a> for it:
</p>

![](./readme/595969d8-909e-438f-9c28-073186fcb598.png)
![](./readme/e46b11bf-6b91-4415-af8a-c9e6169b28fb.png)

<p align="center">
    <a href="https://kartoffels.pwy.io">play it in browser</a> or terminal:<br>
    <code>ssh kartoffels.pwy.io</code>
</p>

## name

kartoffel (🇩🇪)    
= potato    
= slang for "slow machine"    
= exactly what kartoffels are, slow machines indeed!

## building

so you want to launch kartoffels on your own machine? easy!

### using nix

```bash
# step i: clone repository
$ git clone https://github.com/patryk27/kartoffels
$ cd kartoffels

# step ii: create a place for storing kartoffels, server keys etc.
$ mkdir store

# step iii: launch server
$ nix run . -- serve ./store --ssh 127.0.0.1:1314

# step iiii: join the game
$ ssh 127.0.0.1 -p1314
```

note that this only starts the ssh server - if you want to use the web frontend:

```bash
# step iii: launch server (note the `--http` argument)
$ nix run . -- serve ./store --http 127.0.0.1:1313

# step iiii: launch frontend
$ nix run .#web

# step iiiii: join the game at http://localhost:5173
```

### on linux / macos

```bash
# step i: clone repository
$ git clone https://github.com/patryk27/kartoffels
$ cd kartoffels

# step ii: build application
$ cd app
$ cargo build --release
$ cd ..

# step iii: create a place for storing kartoffels, server keys etc.
$ mkdir store

# step iiii: launch server
$ ./app/target/release/kartoffels serve ./store --ssh 127.0.0.1:1314

# step iiiii: join the game
$ ssh 127.0.0.1 -p1314
```

note that this only starts the ssh server - if you want to use the web frontend:

```bash
# step iiii: launch server (note the `--http` argument)
$ ./app/target/release/kartoffels serve ./store --http 127.0.0.1:1313

# step iiiii: launch frontend
$ cd web
$ npm install
$ npm run dev

# step iiiiii: join the game at http://localhost:5173
```

### on windows

i'd suggest installing wsl and following the linux instructions

## license

copyright (c) 2024, patryk wychowaniec (`pwychowaniec @at@ pm.me`).

this program is free software: you can redistribute it and/or modify it under
the terms of the gnu general public license as published by the free software
foundation, version 3.

this program is distributed in the hope that it will be useful, but without any
warranty; without even the implied warranty of merchantability or fitness for a
particular purpose. see the gnu general public license for more details.

you should have received a copy of the gnu general public license along with
this program. if not, see <https://www.gnu.org/licenses/>. 
