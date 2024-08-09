<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <p>let's start with the simplest peripheral: <b>the motor</b></p>
  <p>motor allows to move the robot using the following functions:</p>

  <ul class="compact">
    <li><kbd>motor_step()</kbd>, which moves the robot one tile forward</li>
    <li>
      <kbd>motor_turn(1)</kbd>, which rotates the robot clockwise ("to right")
    </li>
    <li><kbd>motor_turn(-1)</kbd>, which rotates the robot counterclockwise</li>
  </ul>

  <p>since those are just rust functions, you can invoke them like so:</p>

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

  <p style="text-align: right">
    <button @click="ctrl.openTutorialSlide(6)" class="highlighted">
      i'm ready
    </button>
  </p>
</template>
