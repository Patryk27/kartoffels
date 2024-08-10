<script setup lang="ts">
const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <dialog class="game-help">
    <nav>
      <div class="dialog-title">help</div>

      <div class="dialog-buttons">
        <button @click="emit('close')">close</button>
      </div>
    </nav>

    <main>
      <p>hey there soldier and welcome to kartoffels 🫡🫡🫡</p>

      <p>
        the game has a built-in tutorial accessible from the home page, in
        here's just a couple of tips:
      </p>

      <ul class="compact">
        <li>
          <kbd>git clone https://github.com/Patryk27/kartoffel</kbd> to get a
          starting point - see <kbd>README.md</kbd> for building instructions
        </li>

        <li>use the <kbd>[upload bot]</kbd> button to upload the firmware</li>
        <li>navigate map using <kbd>W/A/S/D</kbd> or arrow keys</li>
        <li>bots are represented with the @ char</li>

        <li>
          bots uploaded by you will have
          <span class="inverted">inverted colors</span>
        </li>

        <li>
          use your ide's <kbd>go to definition</kbd> feature to discover how
          robot-specific functions, such as <kbd>radar_scan()</kbd>, work
        </li>
      </ul>
    </main>
  </dialog>
</template>

<style scoped>
.game-help {
  max-width: 768px;
}
</style>
