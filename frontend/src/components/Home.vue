<script setup lang="ts">
import { ref, onMounted, type Ref } from "vue";
import { loadSession, type Session } from "@/logic/State";
import type { ServerGetWorldsResponse, ServerWorld } from "@/logic/Server";

const emit = defineEmits<{
  start: [string, string, string?];
  openHelp: [];
}>();

const worldId = ref<string>(null);
const worlds = ref<ServerWorld[]>(null);
const session: Ref<Session & { worldName?: string }> = ref(loadSession());
const error = ref(null);

function getWorldName(id: string): string {
  if (id == "sandbox") {
    return "sandbox";
  }

  for (const world of worlds.value) {
    if (world.id == id) {
      return world.name;
    }
  }

  throw `unknown world: ${id}`;
}

function handleJoin(worldId: string): void {
  emit("start", worldId, getWorldName(worldId), undefined);
}

function handleRestore(): void {
  emit(
    "start",
    session.value.worldId,
    session.value.worldName,
    session.value.botId,
  );
}

onMounted(async () => {
  try {
    var request = await fetch(`${import.meta.env.VITE_HTTP_URL}/worlds`);
    var response: ServerGetWorldsResponse = await request.json();

    worlds.value = response.worlds;

    if (response.worlds.length > 0) {
      worldId.value = response.worlds[0].id;

      if (session.value) {
        for (const world of response.worlds) {
          if (world.id == session.value.worldId) {
            session.value.worldName = world.name;
          }
        }

        // If the world this session refers to got deleted, no point in
        // offering to restore the session
        if (session.value.worldName == null) {
          session.value = null;
        }
      }
    }
  } catch (err) {
    error.value = err;
  }
});
</script>

<template>
  <main class="home">
    <div class="lead">
      <p>
        welcome to 🥔
        <a href="https://github.com/Patryk27/kartoffels/" target="_blank">
          kartoffels</a
        >
        🥔, an mmo robot combat arena!
      </p>

      <p>
        it's an online game where you implement your own bot and see it fight
        other bots <span class="rainbow">in real time</span>
      </p>

      <p>the rules are simple -- have fun and let the best bot win!</p>

      <p style="text-align: center">
        <img src="./Home/bot.svg" @click="emit('openHelp')" />
      </p>

      <p v-if="error == null" style="text-align: center; padding: 0.5em">
        👉 <a @click="emit('openHelp')">getting started</a> 👈
      </p>

      <p class="quote">
        > senator, you're no kartoffel -- jack kennedy, anno domini 2024
      </p>
    </div>

    <div class="menu">
      <template v-if="error == null">
        <template v-if="worlds == null">loading...</template>

        <template v-else>
          <div class="world-selection">
            <label for="world">choose world and click join:</label>

            <select v-model="worldId">
              <option v-for="world in worlds" :value="world.id">
                {{ world.name }} ({{ world.mode }} + {{ world.theme }})
              </option>

              <option value="sandbox">sandbox (🕵️ private 🕵️)</option>
            </select>

            <button @click="handleJoin(worldId)">join!</button>
          </div>

          <div v-if="worldId == 'sandbox'" class="sandbox-info">
            <p><b>note, soldier!</b></p>

            <p>
              sandbox is a temporary, private world - it is simulated only in
              your browser and so and it disappears once you close or refresh
              the page
            </p>
          </div>

          <div v-if="session" class="session-restore">
            <div class="or">or</div>

            <button @click="handleRestore()">
              restore your previous session
            </button>

            <p>(world: {{ session.worldName }})</p>
          </div>
        </template>
      </template>

      <div v-else>
        <p>
          error: it looks like the server is down, please try again later 👉👈
        </p>

        <p>
          {{ error }}
        </p>
      </div>
    </div>
  </main>
</template>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  max-width: 1024px;
  padding: 1em;

  .lead {
    padding: 1em;
    margin-bottom: 1em;
    border: 1px solid #00ff80;

    p {
      &:first-child {
        margin-top: 0;
      }

      &:last-child {
        margin-bottom: 0;
      }

      &.quote {
        color: #606060;
        text-align: right;
      }
    }
  }

  .menu {
    text-align: center;
  }

  .world-selection {
    margin-top: 1em;

    label {
      display: block;
      margin-bottom: 0.25em;
    }

    button {
      margin-left: 0.5em;
    }
  }

  .sandbox-info {
    margin-top: 1.5em;
    border-top: 1px dashed #444444;
    border-bottom: 1px dashed #444444;

    b {
      color: #ff8000;
    }
  }

  .session-restore {
    margin-top: 1.5em;

    .or {
      margin-bottom: 1.5em;
    }

    button {
      font-weight: 600;
    }

    button + p {
      margin-top: 0.5em;
      color: #606060;
    }
  }

  .rainbow {
    display: inline-block;

    animation:
      rainbow-color 0.5s linear 0s infinite,
      rainbow-rotate 1.8s ease-in-out 0s infinite;
  }
}

@keyframes rainbow-color {
  from {
    color: #6666ff;
  }
  10% {
    color: #0099ff;
  }
  50% {
    color: #00ff00;
  }
  75% {
    color: #ff3399;
  }
  100% {
    color: #6666ff;
  }
}

@keyframes rainbow-rotate {
  from {
    transform: rotate(-2deg);
  }
  50% {
    transform: rotate(2deg);
  }
  to {
    transform: rotate(-2deg);
  }
}
</style>
