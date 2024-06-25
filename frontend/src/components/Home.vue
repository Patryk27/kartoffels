<script setup>
  import { ref, watch, onMounted } from 'vue';
  import { loadSession } from '@/utils/session.ts';

  const emit = defineEmits(['join', 'restore', 'openIntro']);
  const world = ref(null);
  const worlds = ref(null);
  const session = ref(loadSession());
  const error = ref(null);

  onMounted(async () => {
    try {
      var response = await fetch(`${import.meta.env.VITE_HTTP_URL}/worlds`);
      var response = await response.json();

      worlds.value = response.worlds;

      if (response.worlds.length > 0) {
        world.value = response.worlds[0].id;

        for (const world of response.worlds) {
          if (world.id == session.value.worldId) {
            session.value.worldName = world.name;
          }
        }

        // If the world this session refers to got deleted, no point in offering
        // to restore the session
        if (session.value.worldName == null) {
          session.value = null;
        }
      }
    } catch (err) {
      error.value = err;
    }
  });
</script>

<template>
  <main class="component">
    <div class="intro">
      <p>
        welcome to 🥔
        <a href="https://github.com/Patryk27/kartoffels/" target="_blank">
          kartoffels</a>
        🥔, an online robot combat arena!
      </p>

      <p>
        it's an online game where you implement your own bot and see it fight
        other bots <span class="rainbow">in real time</span>
      </p>

      <p>
        the rules are simple -- have fun and let the best bot win!
      </p>

      <p v-if="error == null" style="text-align: center; padding: 0.5em">
        👉 <a @click="emit('openIntro')">getting started</a> 👈
      </p>

      <p class="quote">
        > senator, you're no kartoffel -- jack kennedy, anno domini 2024
      </p>
    </div>

    <div class="menu">
      <template v-if="error == null">
        <template v-if="worlds == null || worlds.length > 0">
          <div class="world-selection">
            <label for="world">
              choose world and click join:
            </label>

            <select v-model="world">
              <option v-for="world in worlds" :value="world.id">
                {{ world.name }} ({{ world.mode }}; {{ world.theme }})
              </option>
            </select>

            <button @click="emit('join', world)">
              join!
            </button>
          </div>

          <div v-if="session != null" class="session-restore">
            <div class="or">
              or
            </div>

            <button @click="emit('restore', session.worldId, session.botId)">
              restore your previous session
            </button>

            <p>
              (world: {{ session.worldName }})
            </p>
          </div>
        </template>

        <div v-else>
          <p>
            it looks like this server has no worlds configured, so unfortunately
            you can't join the game at this moment
          </p>

          <p>
            if you're the administrator of this server, please consult readme.md
          </p>
        </div>
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
  .component {
    display: flex;
    flex-direction: column;
    max-width: 1024px;
    padding: 1em;

    .intro {
      padding: 1em;
      margin-bottom: 1em;
      border: 1px solid #00ff80;
      border-radius: 5px;

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
