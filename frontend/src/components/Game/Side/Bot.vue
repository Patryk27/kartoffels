<script setup>
  import { computed } from 'vue';
  import { botIdToColor } from '@/utils/bot.ts';
  import { durationToHuman } from '@/utils/other.ts';

  const emit = defineEmits([
    'botUpload',
    'botConnect',
    'botDisconnect',
  ]);

  const props = defineProps(['bot', 'paused']);

  const serialStr = computed(() => {
    if (props.bot == null || props.bot.serial == null) {
      return null;
    }

    let out = '';
    let buf = null;

    for (const op of props.bot.serial) {
      switch (op) {
        case 0xffffff00:
          buf = '';
          break;

        case 0xffffff01:
          out = buf ?? '';
          buf = '';
          break;

        case op:
          const ch = String.fromCodePoint(op);

          if (buf === null) {
            out += ch;
          } else {
            buf += ch;
          }
      }
    }

    return out;
  });

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
      <button :disabled="paused"
              @click="paused ? null : handleConnectToBot()">
        connect to bot
      </button>

      <button :disabled="paused"
              @click="paused ? null : handleUploadBot()">
        upload bot
      </button>
    </div>
  </div>

  <div v-else class="game-side-bot">
    <div>
      <button style="width: 100%"
              @click="emit('botDisconnect')">
        disconnect from bot
      </button>

      <p>
        id: <br />

        <span :style="`color: ${botIdToColor(bot.id)}`">
          {{ bot.id }}
        </span>
      </p>

      <template v-if="bot.status == 'alive'">
        <p>
          status: <br />

          <span class="status-alive">
            alive
          </span>

          ({{ durationToHuman(Math.round(bot.age)) }})
        </p>

        <p>
          serial port:
        </p>
      </template>

      <template v-if="bot.status == 'dead'">
        <p>
          status: <br />

          <span class="status-dead">
            dead
          </span>

          <br />

          (since {{ (new Date(bot.killed_at)).toLocaleString() }})
        </p>

        <p>
          message:
        </p>
      </template>

      <template v-else-if="bot.status == 'queued'">
        <p>
          status: <br />

          <span class="status-queued">
            {{ bot.requeued ? 'requeued' : 'queued' }}
            ({{ bot.queue_place + 1 }} / {{ bot.queue_len }})
          </span>
        </p>

        <p>
          message:
        </p>
      </template>
    </div>

    <template v-if="bot.status == 'dead' || bot.status == 'queued'">
      <textarea readonly style="resize: none" :value="bot.msg" />
    </template>

    <template v-if="bot.status == 'alive'">
      <textarea readonly style="resize: none" :value="serialStr" />

      <div>
        <input id="bot-follow"
               type="checkbox"
               v-model="bot.is_followed"
               :disabled="paused" />

        <label for="bot-follow">
          follow with camera
        </label>
      </div>
    </template>
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
