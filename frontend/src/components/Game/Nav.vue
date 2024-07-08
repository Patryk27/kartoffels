<script setup lang="ts">
import type { GameStatus } from "../Game.vue";

const emit = defineEmits<{
  leave: [];
  pause: [];
  openIntro: [];
  openSandboxConfig: [];
}>();

defineProps<{
  worldId: string;
  worldName: string;
  status: GameStatus;
  paused: boolean;
}>();
</script>

<template>
  <nav class="game-nav">
    <div class="game-nav-back">
      <button @click="emit('leave')">go back</button>
    </div>

    <div class="game-nav-world">
      <template v-if="worldId == 'sandbox'">
        <span style="color: #ff8000">🕵️ sandbox 🕵️ </span>
        <button @click="emit('openSandboxConfig')">configure</button>
      </template>

      <template v-else>
        {{ worldName }}
      </template>
    </div>

    <div class="game-nav-control">
      <button :disabled="status == 'reconnecting'" @click="emit('openIntro')">
        help
      </button>

      <button
        :class="{ paused }"
        :disabled="status == 'reconnecting'"
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
      border: 1px solid red;
    }

    & + button {
      margin-left: 0.5em;
    }
  }
}
</style>
