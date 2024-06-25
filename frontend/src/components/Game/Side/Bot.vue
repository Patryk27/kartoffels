<script setup>
  import { botIdToColor } from '@/utils/bot.ts';

  const emit = defineEmits([
    'botUpload',
    'botConnect',
    'botDisconnect',
  ]);

  const props = defineProps(['bot', 'paused']);

  function handleConnectToBot() {
    const id = prompt('enter bot id:');

    if (id) {
      emit('botConnect', id);
    }
  }

  function handleUploadBot() {
    const input = document.createElement('input');

    input.type = 'file';

    input.onchange = event => {
      emit('botUpload', event.target.files[0]);
    };

    input.click();
  }
</script>

<template>
  <div v-if="bot == null" class="game-side-bot">
    <div>
      <button :disabled="paused" @click="paused ? null : handleConnectToBot()">
        connect to bot
      </button>

      <button :disabled="paused" @click="paused ? null : handleUploadBot()">
        upload bot
      </button>
    </div>
  </div>

  <div v-else class="game-side-bot">
    <div>
      <button @click="emit('botDisconnect')" style="width: 100%">
        disconnect from bot
      </button>

      <p>
        id: <br />

        <span :style="`color: ${botIdToColor(bot.id)}`">
          {{ bot.id }}
        </span>
      </p>

      <p>
        serial port:
      </p>
    </div>

    <textarea
      readonly :value="bot.serial"
      style="resize: none" />

    <div>
      <!-- TODO v-model here is invalid, but :value seems not to work -->
      <input
        id="bot-follow"
        type="checkbox"
        v-model="bot.is_followed"
        :disabled="paused" />

      <label for="bot-follow">
        follow with camera
      </label>
    </div>
  </div>
</template>

<style scoped>
  .game-side-bot {
    display: flex;
    flex-direction: column;
    flex-grow: 1;
    padding-bottom: 1em;

    button + button {
      margin-left: 1em;
    }

    p {
      &:first-child {
        margin-top: 0;
      }
    }

    textarea {
      flex-grow: 1;
      margin-bottom: 1em;
    }
  }
</style>