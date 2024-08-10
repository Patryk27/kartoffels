<script setup lang="ts">
import { worlds } from "./SandboxConfig/Presets";

const emit = defineEmits<{
  close: [];
  recreate: [any];
}>();

let worldIdx = 0;
</script>

<template>
  <dialog class="game-sandbox-config" open>
    <nav>
      <div class="dialog-title">sandbox config</div>

      <div class="dialog-buttons">
        <button @click="emit('close')">close</button>
      </div>
    </nav>

    <main>
      <div class="field">
        <label for="world">world preset:</label>

        <select for="world" v-model="worldIdx">
          <option v-for="(world, worldIdx) in worlds" :value="worldIdx">
            {{ world.name }}
          </option>
        </select>
      </div>
    </main>

    <footer style="text-align: right">
      <button @click="emit('recreate', worlds[worldIdx].config)">
        recreate sandbox
      </button>
    </footer>
  </dialog>
</template>

<style scoped>
.game-sandbox-config {
  .field {
    margin-bottom: 1em;

    label {
      display: block;
      margin-bottom: 0.2em;
    }

    select {
      display: block;
      width: 100%;
    }
  }
}
</style>
