<script setup lang="ts">
const emit = defineEmits<{
  leave: [];
}>();
</script>

<template>
  <main class="intro">
    <p>
      hey there solder, you great piece of low melting point metal alloy 🫡🫡🫡
    </p>

    <p>
      in kartoffels you're the lead programmer responsible for leading your bot
      into the victory - in this house:
    </p>

    <p>
      - maps are two-dimensional
      <br />
      - robots contain peripherals that your code controls
      <br />
      - codes are executed on emulated RISC-V CPUs clocked at 64 KHz
      <br />
      - the game is simulated on a central server -- let the meritocracy thrive
    </p>

    <p>now, much too much talk, grug wants start:</p>

    <pre>
      $ git clone https://github.com/patryk27/kartoffel
      $ cd kartoffel
      $ ./build</pre
    >

    <p>
      ... and then go back, select any map, upload <kbd>./kartoffel</kbd> and
      have fun!
    </p>

    <p>
      in-game controls boil down to arrow keys (or WASD) and the mouse, nothing
      fancy
    </p>

    <p class="note">
      <b>note, soldier!</b> if you want to practice without actually submitting
      your bot yet, choose the <kbd>sandbox</kbd> map - it is simulated in your
      browser and allows to debug bots etc.
    </p>

    <div style="text-align: center; padding: 2em 0">
      <button @click="emit('leave')">
        i'm ready<span style="color: #ff0000">*</span>
      </button>
    </div>

    <p style="color: #505050; font-style: italic; text-align: right">
      <span style="color: #ff0000">*</span> kartoffels ltd is not responsible
      and can't be held liable for loss of hearing, loss of sight, spilling all
      your matcha onto floor, sudden feeling of the flight and fight syndrome,
      wanting to do origami but being unable to etc. etc. etc.
    </p>
  </main>
</template>

<style scoped>
.intro {
  max-width: 1024px;
  padding: 1em;
  border: 1px solid #ff8000;
  border-radius: 5px;

  p {
    &:first-child {
      margin-top: 0;
    }

    &:last-child {
      margin-bottom: 0;
    }

    &.note {
      b {
        color: #ff8000;
      }
    }
  }
}
</style>
