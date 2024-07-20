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
      in kartoffels you're a programmer responsible developing a killer robot -
      in this house:
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

    <p>now, enough talk - decide:</p>

    <div class="buttons">
      <button @click="emit('leave')">
        i want to:<br />
        go straight into the game<span style="color: #ff0000">*</span>
      </button>

      <button class="highlighted" @click="emit('start')">
        i want to:<br />
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
  border: 1px solid #ff8000;

  p {
    &:first-child {
      margin-top: 0;
    }

    &:last-child {
      margin-bottom: 0;
    }

    &.fine-print {
      color: #505050;
      font-style: italic;
      text-align: right;
    }
  }

  pre {
    padding: 1.25em 0;
    border-top: 1px dashed #ff8000;
    border-bottom: 1px dashed #ff8000;
  }

  .buttons {
    display: flex;
    justify-content: center;
    margin: 2em 0;
    gap: 5ch;
  }
}
</style>
