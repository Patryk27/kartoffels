<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.onSlide(5, () => {
  ctrl.waitFor("tutorial.continue").then(() => {
    ctrl.openSlide(6);
  });
});
</script>

<template>
  <p>let's start with the simplest peripheral: <b>the motor</b></p>
  <p>motor allows to move the robot using the following functions:</p>

  <ul>
    <li><kbd>motor_step()</kbd>, which moves the robot one tile forward</li>
    <li>
      <kbd>motor_turn(1)</kbd>, which rotates the robot clockwise ("to right")
    </li>
    <li><kbd>motor_turn(-1)</kbd>, which rotates the robot counterclockwise</li>
  </ul>

  <p>since those are just rust functions, you can invoke them like so:</p>

  <pre>
#[no_mangle]
fn main() {
    loop {
        motor_step();
    }
}</pre
  >

  <p>okie, now:</p>

  <ul>
    <li>copy this code into <kbd>src/main.rs</kbd></li>
    <li>
      build the project (by running <kbd>./build</kbd> or
      <kbd>./build.bat</kbd>)
    </li>
    <li>lemme know once you're ready!</li>
  </ul>

  <p>
    don't forget to leave the other parts of code (such as
    <kbd>#![no_std]</kbd> or <kbd>use kartoffel::*;</kbd>) in place - they are
    required, it's just that for brevity the snippets in here will not mention
    them anymore
  </p>

  <p style="text-align: right">
    <button @click="ctrl.emit('tutorial.continue')" class="highlighted">
      i'm ready
    </button>
  </p>
</template>
