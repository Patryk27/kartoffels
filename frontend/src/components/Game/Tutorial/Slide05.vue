<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <main>
    <p>let's start with the simplest peripheral, <b>the motor</b></p>

    <p>
      motor allows for the robot to move (d'oh!) and it can be controlled using
      these three functions:
    </p>

    <ul class="compact">
      <li>
        <kbd>motor_step()</kbd>, which moves the robot one tile forward in the
        direction the robot is currently facing
      </li>
      <li>
        <kbd>motor_turn_left()</kbd>, which rotates the robot counterclockwise
      </li>
      <li><kbd>motor_turn_right()</kbd>, which rotates the robot clockwise</li>
    </ul>

    <p>
      putting it to practice, you can make the robot move forward by repeatedly
      calling <kbd>motor_step()</kbd>, like so:
    </p>

    <pre>
#![no_std]
#![no_main]

use kartoffel::*;

#[no_mangle]
fn main() {
    loop {
        motor_step();
    }
}</pre
    >

    <p>okie, now:</p>

    <ul class="compact">
      <li>copy the code above into <kbd>src/main.rs</kbd></li>
      <li>
        build the project (by running <kbd>./build</kbd> or
        <kbd>./build.bat</kbd>)
      </li>
      <li>lemme know once you're ready!</li>
    </ul>
  </main>

  <footer style="text-align: right">
    <button @click="ctrl.openTutorialSlide(6)" class="highlighted">
      i'm ready
    </button>
  </footer>
</template>
