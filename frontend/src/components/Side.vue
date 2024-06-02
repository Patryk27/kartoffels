<script setup>
  import { ref, onMounted, watch, computed } from 'vue';

  const emit = defineEmits([
    'botUpload',
    'botDisconnect',
    'botFollow',
    'botClick',
  ]);

  const props = defineProps(['mode', 'bot', 'bots', 'paused']);

  function connectToBot() {
    /* TODO */
  }

  function uploadBot() {
    const input = document.createElement('input');

    input.type = 'file';

    input.onchange = event => {
      emit('botUpload', event.target.files[0]);
    };

    input.click();
  }

  function durationToHuman(sec) {
    if (sec < 60) {
      return `${sec}s`;
    } else {
      const mins = Math.floor(sec / 60);
      const secs = sec % 60;

      return `${mins}m ${secs}s`;
    }
  }
</script>

<template>
  <div id="side">
    <div v-if="bot == null" id="bot">
      <div>
        <button :disabled="paused" @click="paused ? null : connectToBot()">
          connect to bot
        </button>

        <button :disabled="paused" @click="paused ? null : uploadBot()">
          upload bot
        </button>
      </div>
    </div>

    <div v-else id="bot">
      <div>
        <button @click="emit('botDisconnect')" style="width: 100%">
          disconnect from bot
        </button>

        <p>
          id: <br />
          {{ bot.id }}
        </p>

        <p>
          <!-- TODO v-model here is invalid, but :value seems not to work -->
          <input
            id="bot-follow"
            type="checkbox"
            v-model="bot.is_followed"
            @click="ev => emit('botFollow', ev.target.checked)"/>

          <label for="bot-follow">
            follow with camera
          </label>
        </p>

        <p>
          uart:
        </p>
      </div>

      <textarea readonly :value="bot.uart" />
    </div>

    <div v-if="bots != null" id="bots">
      <table>
        <thead>
          <tr>
            <th>nth</th>
            <th>bot</th>
            <th>score</th>
          </tr>
        </thead>

        <tbody>
          <tr
            v-for="nth in 8"
            :key="nth"
            :set="botItem = bots[nth - 1]">
            <template v-if="botItem">
              <td>
                #{{ nth }}
              </td>

              <td>
                <a @click="emit('botClick', bots[nth - 1].id)">
                  {{ botItem.id }}
                </a>
              </td>

              <td style="text-align: right">
                {{ mode.scores[botItem.id] ?? '0' }}
              </td>
            </template>

            <template v-else>
              <td>
                -
              </td>

              <td style="text-align: center">
                -
              </td>

              <td style="text-align: right">
                -
              </td>
            </template>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
  #side {
    display: flex;
    flex-direction: column;
    margin-left: 1em;
  }

  #bot {
    display: flex;
    flex-direction: column;
    flex-grow: 1;

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

  #mode, #bots {
    border-top: 1px solid #444444;
    padding-top: 1em;
  }

  #mode {
    padding-top: 1em;
    padding-bottom: 1em;

    p {
      margin-top: 0;
      margin-bottom: 0;

      padding-top: 0;
      padding-bottom: 0;
    }
  }

  #bots {
    table {
      th {
        font-weight: normal;
      }

      td:nth-child(2) {
        text-align: right;
      }
    }
  }
</style>
