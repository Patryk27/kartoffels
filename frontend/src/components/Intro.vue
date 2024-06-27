<script setup>
  const emit = defineEmits(['leave']);
</script>

<template>
  <main class="intro">
    <p>
      hey there solder, you great piece of low melting point metal alloy
      🫡🫡🫡
    </p>

    <p>
      in kartoffels you're the lead programmer responsible for leading your
      bot into the victory - in this house:
    </p>

    <p>
      - maps are two-dimensional and vary in type (arena, dungeon etc.)
      <br />
      - robots contain peripherals (radar, motors etc.) that your code
        (aka firmware) controls
      <br />
      - firmwares run on emulated RISC-V CPUs clocked at 64 KHz
      <br />
      - game is simulated on a central server and it's anonymous - let the
        meritocracy thrive
    </p>

    <p>
      ---
    </p>

    <p>
      now, much too much talk, grug wants start:
    </p>

    <pre>
      $ git clone github.com/patryk27/kartoffels-bot
      $ cd kartoffels-bot
      $ cargo build --release</pre>

    <p>
      then go back, select any map, upload
      <i>`target/release/riscv64-unknown-bot/bot`</i> and play with the code!
    </p>

    <p>
      (( in-game controls boil down to arrow keys (or WASD) and the mouse ))
    </p>

    <div style="text-align: center; padding: 2.0em 0">
      <button @click="emit('leave')">
        so help me dog i'm ready<span style="color: #ff0000">*</span>
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
    }
  }
</style>
