<script setup lang="ts">
import type { GameController } from "./Controller";
import type { GameWorld } from "./State";

const emit = defineEmits<{
  leave: [];
  pause: [];
  openHelp: [];
  openConfig: [];
}>();

defineProps<{
  ctrl: GameController;
  world: GameWorld;
  paused: boolean;
}>();
</script>

<template>
  <nav class="game-nav">
    <div class="game-nav-back">
      <button @click="emit('leave')">leave</button>
    </div>

    <div class="game-nav-world">
      <template v-if="world.id == 'sandbox'">
        <span style="color: var(--orange)">🕵️ sandbox 🕵️ </span>
        <button @click="emit('openConfig')">configure</button>
      </template>

      <template v-else>
        {{ world.name }}
      </template>
    </div>

    <div class="game-nav-control">
      <button
        :disabled="
          world.costatusalue == 'reconnecting' || ctrl.ui.value.btnHelpDisabled
        "
        @click="emit('openHelp')"
      >
        help
      </button>

      <button
        :class="{ paused }"
        :disabled="
          world.costatusalue == 'reconnecting' || ctrl.ui.value.btnPauseDisabled
        "
        @click="emit('pause')"
      >
        <template v-if="paused">resume</template>
        <template v-else>pause</template>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.game-nav {
  display: flex;
  margin-bottom: 0.5em;

  .game-nav-back,
  .game-nav-world {
    flex-grow: 1;
  }

  .game-nav-world {
    align-self: center;
  }

  button {
    &.paused {
      border: 1px solid var(--red);
    }

    & + button {
      margin-left: 0.5em;
    }
  }
}
</style>
