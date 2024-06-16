<script setup>
  import { ref, onMounted, watch, computed } from 'vue';

  const emit = defineEmits([
    'botUpload',
    'botDisconnect',
    'botFollow',
    'botClick',
  ]);

  const props = defineProps(['mode', 'bot', 'bots', 'paused']);

  const botEntries = computed(() => {
    let allEntries = [];

    for (const [id, bot] of Object.entries(props.bots ?? { })) {
      allEntries.push({
        id,
        score: (props.mode ?? { }).scores[id] ?? 0,
      });
    }

    allEntries.sort((a, b) => {
      if (a.score == b.score) {
        return b.id.localeCompare(a.id);
      } else {
        return b.score - a.score;
      }
    });

    // ---

    let entries = [];

    for (let nth = 0; nth < 8; nth += 1) {
      const entry = allEntries[nth];

      if (entry) {
        entries.push({
          nth: nth + 1,
          ...entry
        });
      } else {
        entries.push(null);
      }
    }

    if (props.bot != null) {
      const connectedBotNth = allEntries.findIndex((entry) => {
        return entry.id == props.bot.id;
      });

      if (connectedBotNth >= entries.length) {
        entries[entries.length - 1] = {
          nth: connectedBotNth + 1,
          ...allEntries[connectedBotNth]
        };
      }
    }

    return entries;
  });

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
          uart:
        </p>
      </div>

      <textarea readonly :value="bot.uart" />

      <div>
        <!-- TODO v-model here is invalid, but :value seems not to work -->
        <input
          id="bot-follow"
          type="checkbox"
          v-model="bot.is_followed"
          @click="ev => emit('botFollow', ev.target.checked)"
          :disabled="paused" />

        <label for="bot-follow">
          follow with camera
        </label>
      </div>
    </div>

    <div id="bots">
      <table>
        <thead>
          <tr>
            <th colspan="2">bot</th>
            <th>score</th>
          </tr>
        </thead>

        <tbody>
          <tr v-for="botEntry in botEntries">
            <template v-if="botEntry">
              <td>
                #{{ botEntry.nth }}&nbsp;
              </td>

              <td>
                <a @click="emit('botClick', botEntry.id)"
                   :class="botEntry.id == bot?.id ? 'connected-bot' : ''">
                  {{ botEntry.id }}
                </a>
              </td>

              <td>
                {{ botEntry.score }}
              </td>
            </template>

            <template v-else>
              <td>
                -
              </td>

              <td style="text-align: center">
                -
              </td>

              <td>
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

      td:nth-child(3) {
        text-align: right;
      }
    }

    a {
      color: rgb(0, 255, 128);

      &.connected-bot {
        color: rgb(0, 128, 255);
        font-weight: bold;
      }
    }
  }
</style>
