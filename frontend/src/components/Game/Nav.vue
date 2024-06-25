<script setup>
  const emit = defineEmits(['leave', 'pause', 'openTutorial']);
  const props = defineProps(['paused']);
</script>

<template>
  <nav class="game-nav">
    <div>
      <button @click="emit('leave')">
        leave world
      </button>
    </div>

    <div>
      <button @click="emit('openTutorial')">
        help
      </button>

      <button
          class="pause"
          :class="paused ? 'paused' : ''"
          @click="emit('pause')">
        <template v-if="paused">
          resume
        </template>

        <template v-else>
          pause
        </template>
      </button>
    </div>
  </nav>
</template>

<style scoped>
  .game-nav {
    display: flex;
    margin-bottom: 0.5em;

    >div:first-child {
      flex-grow: 1;
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
