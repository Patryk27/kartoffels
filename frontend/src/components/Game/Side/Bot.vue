<script setup lang="ts">
import { computed } from "vue";
import { botIdToColor } from "@/utils/bot";
import { durationToHuman, ordinal } from "@/utils/other";
import type { GameCtrl } from "../Ctrl";
import type { GameWorld } from "../World";

const emit = defineEmits<{
  botCreate: [File];
  botCreatePrefab: [string];
  botJoin: [string];
  botLeave: [];
  botDestroy: [];
  botRestart: [];
}>();

const props = defineProps<{
  ctrl: GameCtrl;
  world: GameWorld;
  paused: boolean;
}>();

const serial = computed(() => {
  const bot = props.world.bot.value;

  if (bot.status != "alive") {
    return null;
  }

  let out = "";
  let buf = null;

  for (const op of bot.serial) {
    switch (op) {
      case 0xffffff00:
        buf = "";
        break;

      case 0xffffff01:
        out = buf ?? "";
        buf = "";
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

const events = computed(() => {
  const bot = props.world.bot.value;
  const now = new Date();

  if (bot.status != "alive" && bot.status != "dead" && bot.status != "queued") {
    return "";
  }

  let out = "";

  for (const event of bot.events) {
    let eventHappenedToday =
      event.at.getFullYear() == now.getFullYear() &&
      event.at.getMonth() == now.getMonth() &&
      event.at.getDay() == now.getDay();

    let eventAt = eventHappenedToday
      ? event.at.toLocaleTimeString()
      : event.at.toLocaleString();

    out += `> ${eventAt}\n${event.msg}\n`;
    out += "\n";
  }

  return out;
});

function handleJoinBot() {
  const id = prompt("bot id to connect to:");

  if (id) {
    emit("botJoin", id.trim());
  }
}

function handleCreateBot() {
  const input = document.createElement("input");

  input.type = "file";

  input.onchange = (event) => {
    if (event.target instanceof HTMLInputElement) {
      emit("botCreate", event.target.files[0]);
    }
  };

  input.click();
}
</script>

<template>
  <div v-if="world.bot.value == null" class="game-side-bot">
    <div class="buttons">
      <div class="buttons-row">
        <button
          :disabled="paused || !ctrl.ui.value.enableConnectToBot"
          @click="handleJoinBot"
        >
          connect to bot
        </button>

        <button
          :disabled="paused || !ctrl.ui.value.enableUploadBot"
          :class="{ highlighted: ctrl.ui.value.highlightUploadBot }"
          @click="handleCreateBot"
        >
          upload bot
        </button>
      </div>

      <div v-if="world.id == 'sandbox'" class="buttons-row">
        <button
          :disabled="paused"
          @click="emit('botCreatePrefab', 'roberto')"
          title="roberto is a built-in moderately challenging bot written by kartoffels' author"
        >
          spawn roberto
        </button>
      </div>
    </div>
  </div>

  <div v-else class="game-side-bot">
    <div class="buttons">
      <div class="buttons-row">
        <button
          :disabled="!ctrl.ui.value.enableDisconnectFromBot"
          @click="emit('botLeave')"
        >
          disconnect from bot
        </button>
      </div>

      <div v-if="world.id == 'sandbox'" class="buttons-row">
        <button :disabled="paused" @click="emit('botDestroy')">
          destroy bot
        </button>

        <button :disabled="paused" @click="emit('botRestart')">
          restart bot
        </button>
      </div>
    </div>

    <p>
      id:
      <br />
      <span :style="`color: ${botIdToColor(world.bot.value.id)}`">
        {{ world.bot.value.id }}
      </span>
    </p>

    <template v-if="world.bot.value.status == 'alive'">
      <p>
        status:
        <br />
        <span class="status-alive">alive</span>
        ({{ durationToHuman(Math.round(world.bot.value.age)) }})
      </p>

      <p>serial port:</p>

      <textarea
        v-if="world.bot.value.status == 'alive'"
        :value="serial"
        readonly
      />
    </template>

    <p v-if="world.bot.value.status == 'dead'">
      status:
      <br />
      <span class="status-dead">dead</span>
    </p>

    <p v-else-if="world.bot.value.status == 'queued'">
      status:
      <br />
      <span class="status-queued">
        {{ world.bot.value.requeued ? "requeued" : "queued" }}
        ({{ world.bot.value.place }}{{ ordinal(world.bot.value.place) }})
      </span>
    </p>

    <template v-if="events.length > 0">
      <p>history:</p>
      <textarea readonly :value="events" />
    </template>

    <div v-if="world.bot.value.status == 'alive'">
      <input
        id="bot-follow"
        type="checkbox"
        v-model="world.bot.value.following"
        :disabled="paused"
      />

      <label for="bot-follow"> follow with camera</label>
    </div>
  </div>
</template>

<style scoped>
.game-side-bot {
  display: flex;
  width: 32ch;
  flex-grow: 1;
  flex-direction: column;

  .buttons {
    margin-bottom: var(--text-margin);

    .buttons-row {
      display: flex;

      & + .buttons-row {
        margin-top: 0.5em;
      }

      button {
        display: block;
        width: 100%;
        flex-grow: 1;

        & + button {
          margin-left: 1em;
        }
      }
    }
  }

  p {
    & + p {
      margin-top: 0;
    }
  }

  textarea {
    flex-grow: 1;
    margin-bottom: 1em;
  }

  textarea + p {
    margin-top: 0;
  }

  p:has(+ textarea) {
    margin-bottom: 0.4em;
  }
}
</style>
