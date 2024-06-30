<script setup>
  const emit = defineEmits(['leave', 'pause', 'openIntro']);
  const props = defineProps(['status', 'paused']);
</script>

<template>
  <nav class="game-nav">
    <div>
      <button @click="emit('leave')">
        go back
      </button>
    </div>

    <div>
      <button :disabled="status == 'reconnecting'"
              @click="emit('openIntro')">
        help
      </button>

      <button :class="{ paused }"
              :disabled="status == 'reconnecting'"
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
