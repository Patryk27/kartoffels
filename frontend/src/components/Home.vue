<script setup lang="ts">
import { ref, onMounted, type Ref } from "vue";
import { loadSession, type Session } from "@/logic/State";
import type { ServerGetWorldsResponse, ServerWorld } from "@/logic/Server";

const emit = defineEmits<{
  start: [string, string, string?];
}>();

const world = ref<string>(null);
const worlds = ref<ServerWorld[]>(null);
const session: Ref<Session & { worldName?: string }> = ref(loadSession());
const error = ref(null);

const testimonials = [
  "senator, you're no kartoffel! -- jack kennedy, 2024",
  "amazing piece of technology! -- lewis carroll, 2024",
  "super fun! -- the pope, 2024",
];

const testimonial =
  testimonials[Math.floor(Math.random() * testimonials.length)];

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
  emit("start", worldId, getWorldName(worldId), null);
}

function handleRestore(): void {
  emit(
    "start",
    session.value.worldId,
    session.value.worldName,
    session.value.botId,
  );
}

function log(...data: any[]) {
  console.log("[home]", ...data);
}

onMounted(async () => {
  try {
    log("fetching worlds");

    var request = await fetch(`${import.meta.env.VITE_HTTP_URL}/worlds`);
    var response: ServerGetWorldsResponse = await request.json();

    worlds.value = response.worlds;

    if (response.worlds.length > 0) {
      world.value = response.worlds[0].id;

      if (session.value) {
        log("loading session");

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
    <div class="intro">
      <p>
        welcome to 🥔
        <a href="https://github.com/Patryk27/kartoffels/" target="_blank">
          kartoffels</a
        >
        🥔, a game where everyone's killed and everyone's the killer!
      </p>

      <p>
        this is an mmo arena where you're given a robot with <b>motors</b>,
        <b>a radar</b> and <b>an arm</b>:
      </p>

      <p class="bot">
        <img src="./Home/bot.svg" />
      </p>

      <p>
        ... and your objective is to implement <b>a firmware</b> controlling it
        - developing the best, the longest surviving, the most deadly machine
        imaginable, in rust
      </p>

      <p>
        robots are limited to <b>64 khz cpu</b> and <b>128 kb of ram</b>, and
        the game happens entirely online - you can see your bot fighting bots
        submitted from other players and you can learn from their behavior
      </p>

      <p>feeling up to the challenge?</p>

      <p class="tutorial" @click="emit('start', 'tutorial', 'tutorial')">
        <span class="hand">👉 </span>
        <a href="#">getting started</a>
        <span class="hand"> 👈</span>
      </p>

      <p class="testimonial">> {{ testimonial }}</p>
    </div>

    <div v-if="!error" class="menu">
      <template v-if="worlds == null">loading...</template>

      <template v-else>
        <div class="world-selection">
          <label for="world">choose world and click join:</label>

          <select v-model="world">
            <option v-for="world in worlds" :value="world.id">
              {{ world.name }} ({{ world.mode }} + {{ world.theme }})
            </option>

            <option value="sandbox">🕵️ private sandbox 🕵️</option>
          </select>

          <button @click="handleJoin(world)">join!</button>
        </div>

        <div v-if="world == 'sandbox'" class="sandbox-info">
          <p><b>note, soldier!</b></p>

          <p>
            sandbox is a special world simulated locally in your browser - it
            disappears once you close or refresh the game
          </p>
        </div>

        <div v-if="session" class="session-restore">
          <div class="or">-- or --</div>

          <button @click="handleRestore()">
            restore your previous session <br />
            <span style="color: var(--gray)">({{ session.worldName }})</span>
          </button>
        </div>
      </template>
    </div>

    <div v-if="error" class="error">
      <p>whoopsie, it looks like the server is down:</p>

      <p>
        {{ error }}
      </p>
    </div>
  </main>
</template>

<style scoped>
.home {
  display: flex;
  flex-direction: column;
  max-width: 1024px;
  padding: 1em;

  .intro {
    padding: 1em;
    margin-bottom: 1em;
    border: 1px solid var(--green);

    p {
      &:first-child {
        margin-top: 0;
      }

      &:last-child {
        margin-bottom: 0;
      }

      &.bot {
        text-align: center;
        margin-top: 0.8em;
        margin-bottom: 0;
        padding: 0.5em 0;

        img {
          animation: bot-anim 2s ease-in-out 0s infinite;
        }
      }

      &.tutorial {
        cursor: pointer;
        padding: 0.33em 0;
        text-align: center;

        &:hover {
          a {
            text-decoration: underline;
          }
        }

        .hand {
          position: relative;
          top: 0.12em;
          animation: hand-anim 1s ease-in-out 0s infinite;
        }
      }

      &.testimonial {
        color: var(--gray);
        text-align: right;
      }
    }
  }

  .menu {
    text-align: center;

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
      padding-top: var(--text-margin);
      border-top: 1px dashed var(--gray);
      border-bottom: 1px dashed var(--gray);

      b {
        color: var(--orange);
      }
    }

    .session-restore {
      margin-top: 1.5em;

      .or {
        margin-bottom: 1.5em;
      }

      button {
        span {
          display: inline-block;
          margin-top: 0.25em;
        }
      }
    }
  }

  .error {
    padding: 1em;
    border: 1px dashed var(--red);
    text-align: center;

    p {
      &:last-child {
        margin-bottom: 0;
      }
    }
  }
}

@keyframes bot-anim {
  from {
    transform: rotate(-4deg);
  }
  50% {
    transform: rotate(8deg);
  }
  to {
    transform: rotate(-4deg);
  }
}

@keyframes hand-anim {
  from {
    margin-left: 0;
    margin-right: 0;
  }
  50% {
    margin-left: 0.75ch;
    margin-right: 0.75ch;
  }
  to {
    margin-left: 0;
    margin-right: 0;
  }
}
</style>
