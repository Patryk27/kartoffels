<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <main>
    <p>
      you see, most robot-specific functions, such as
      <kbd>radar_scan_...()</kbd> or <kbd>motor_step()</kbd>, can only be called
      every now and then
    </p>

    <p>
      for instance the <kbd>motor_step()</kbd> function has cooldown of 20k
      ticks, which means that it can be called at most
      <kbd>64000 / 20000 = 3</kbd> times a second¹ - invoking the function
      before this time has passed will do nothing
    </p>

    <p>(( ¹each kartoffel has a 64 KHz CPU ))</p>

    <p>in practice, it means that if you call a function twice in a row:</p>

    <pre>
motor_step();
motor_step();</pre
    >

    <p>
      ... the robot will move only one tile - the second step() will do nothing,
      because the motor is not ready to accept another command
    </p>

    <p>the correct way to move two tiles could be:</p>

    <pre>
motor_wait();
motor_step();

motor_wait();
motor_step();</pre
    >

    <p>
      each peripheral has its own cooldow , which you can read more about when
      you click <kbd>go to definition</kbd> (or whatever your ide has) on
      <kbd>motor_step()</kbd> and the like
    </p>
  </main>

  <footer style="text-align: right">
    <button class="highlighted" @click="ctrl.openTutorialSlide(13)">
      yes that makes sense
    </button>
  </footer>
</template>
