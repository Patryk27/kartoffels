<script setup lang="ts">
const emit = defineEmits<{
  leave: [];
  start: [];
}>();
</script>

<template>
  <main class="help">
    <p>
      hey there solder, you great piece of low-melting-point metal alloy 🫡🫡🫡
    </p>

    <p>
      in kartoffels you're the lead programmer developing a killer robot - in
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
      - the game is simulated on a central server -- let the meritocracy thrive!
    </p>

    <p>now, enough talk - decide:</p>

    <div class="buttons">
      <button @click="emit('leave')">
        go straight into the game<span style="color: #ff0000">*</span>
      </button>

      <button class="highlighted" @click="emit('start')">
        start tutorial<span style="color: #ff0000">*</span>
      </button>
    </div>

    <p class="fine-print">
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
  border: 1px solid var(--orange);

  p {
    &:first-child {
      margin-top: 0;
    }

    &:last-child {
      margin-bottom: 0;
    }

    &.fine-print {
      color: var(--gray);
      padding-top: var(--text-margin);
      border-top: 1px dashed var(--gray);
      font-style: italic;
      text-align: right;
    }
  }

  .buttons {
    display: flex;
    justify-content: space-around;
    margin: 2em 0;

    button {
      width: 32ch;
      height: 2.5em;
    }
  }
}
</style>
