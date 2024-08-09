<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <p>
    you see, most robot-specific functions, such as
    <kbd>radar_scan_...()</kbd> or <kbd>motor_step()</kbd>, can only be called
    every now and then
  </p>

  <p>
    for instance the <kbd>motor_step()</kbd> function has a cooldown of 20_000
    ticks, which means that it can be called only <kbd>64_000 / 20_000 = 3</kbd>
    times a second - invoking the function before the time has passed will make
    it a no-op (i.e. do nothing)
  </p>

  <p>in practice, it means that if you call a function twice in a row:</p>

  <pre>
motor_step();
motor_step();</pre
  >

  <p>
    ... the robot will only move one tile, the second stepping will do nothing,
    because the motor is not ready to accept another command
  </p>

  <p>the correct way to step two tiles would be:</p>

  <pre>
motor_wait();
motor_step();

motor_wait();
motor_step();</pre
  >

  <p>
    each peripheral has its own cooldown time, which you can read more about
    when you click <kbd>go to definition</kbd> (or whatever your ide has) on
    <kbd>motor_step()</kbd> and the other functions
  </p>

  <footer style="text-align: right">
    <button class="highlighted" @click="ctrl.openTutorialSlide(13)">
      yes that makes sense
    </button>
  </footer>
</template>
