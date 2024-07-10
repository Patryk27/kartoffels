<script setup lang="ts">
const emit = defineEmits<{
  leave: [];
}>();
</script>

<template>
  <main class="help">
    <p>
      hey there solder, you great piece of low melting point metal alloy 🫡🫡🫡
    </p>

    <p>
      in kartoffels you're programmer leading your robot into the victory - in
      this house:
    </p>

    <p>
      - maps are two-dimensional
      <br />
      - bots contain peripherals (motor, radar & arm) that your code controls
      <br />
      - your code runs on an emulated RISC-V CPU clocked at 64 KHZ (+ you've got
      128 KB RAM)
      <br />
      - the game is simulated on a central server -- let the meritocracy thrive
    </p>

    <p>
      the purpose of your robot is, of course, to survive and kill as many enemy
      robots as possible - and everybody's an enemy
    </p>

    <p>
      now, enough talk - start by following this carefully crafted instruction:
    </p>

    <pre>
      $ git clone https://github.com/patryk27/kartoffel
      $ cd kartoffel
      $ ./build</pre
    >

    <p>
      ... and then go back, select any map and upload <kbd>./kartoffel</kbd> -
      have fun & improve!
    </p>

    <p>in-game controls boil down to arrow keys / WASD, and mouse</p>

    <p class="note">
      <b>note, soldier!</b> if you want to practice without submitting your bot
      just yet, choose the <kbd>sandbox</kbd> map - it is simulated in your
      browser and provides extra debugging facilities
    </p>

    <div style="text-align: center; padding: 2em 0">
      <button @click="emit('leave')">
        i'm ready, go back<span style="color: #ff0000">*</span>
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
.help {
  max-width: 1024px;
  padding: 1em;
  border: 1px solid #ff8000;

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

  pre {
    margin: 1em 0;
    padding: 1em 0;
    border-top: 1px dashed #ff8000;
    border-bottom: 1px dashed #ff8000;
  }
}
</style>
