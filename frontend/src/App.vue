<script setup>
  import { ref, onMounted } from 'vue';
  import Canvas from './components/Canvas.vue';
  import Nav from './components/Nav.vue';
  import Side from './components/Side.vue';

  const map = ref(null);
  const mode = ref(null);
  const bot = ref(null);
  const bots = ref(null);
  const camera = ref(null);
  const paused = ref(false);
  const crash = ref(null);

  let session = null;

  onMounted(() => {
    document.onkeydown = () => {
      const moveCamera = (dx, dy) => {
        if (camera.value != null) {
          camera.value.x += dx;
          camera.value.y += dy;

          if (bot.value != null) {
            bot.value.is_followed = false;
          }
        }
      };

      switch (event.keyCode) {
        case 37: // left
        case 65: // a
          moveCamera(-8, 0);
          break;

        case 38: // up
        case 87: // w
          moveCamera(0, -8);
          break;

        case 39: // right
        case 68: // d
          moveCamera(8, 0);
          break;

        case 40: // down
        case 83: // s
          moveCamera(0, 8);
          break;

        case 32:
          togglePause();
      }
    };

    window.onerror = (msg) => {
      crash.value = msg;
    };
  });

  function join(newWorldId, newBotId) {
    if (session != null) {
      if (session.socket != null) {
        session.socket.close();
      }

      session = null;
    }

    localStorage.removeItem('worldId');
    localStorage.removeItem('botId');

    const socketUrl =
      (newBotId == null)
        ? `${import.meta.env.VITE_WS_URL}/worlds/${newWorldId}`
        : `${import.meta.env.VITE_WS_URL}/worlds/${newWorldId}/bots/${newBotId}`;

    session = {
      worldId: newWorldId,
      botId: newBotId,
      socket: new WebSocket(socketUrl),
    };

    // ---

    map.value = null;
    mode.value = null;
    bot.value = null;
    bots.value = null;
    camera.value = null;
    paused.value = false;

    // ---

    session.socket.onopen = () => {
      localStorage.setItem('worldId', session.worldId);

      if (session.botId == null) {
        bot.value = null;
      } else {
        localStorage.setItem('botId', session.botId);

        bot.value = {
          id: newBotId,
          is_followed: true,
        };
      }
    };

    session.socket.onmessage = event => {
      const data = JSON.parse(event.data);

      if (data.map != null) {
        map.value = {
          size: data.map.size,
          tiles: data.map.tiles,
          bots: [],
        };

        camera.value = {
          x: Math.round(data.map.size[0] / 2),
          y: Math.round(data.map.size[1] / 2),
        };
      }

      if (data.mode != null) {
        mode.value = data.mode;
      }

      if (data.bots != null) {
        let mapBots = [];

        for (const [botId, bot] of Object.entries(data.bots)) {
          const tileIdx = bot.pos[1] * map.value.size[0] + bot.pos[0];

          mapBots[tileIdx] = {
            id: botId,
          };
        }

        bots.value = data.bots;
        map.value.bots = mapBots;

        if (bot.value != null && bot.value.is_followed) {
          const botEntry = data.bots[bot.value.id];

          if (botEntry != null) {
            camera.value = {
              x: botEntry.pos[0],
              y: botEntry.pos[1],
            };
          }
        }
      }

      if (bot.value != null) {
        bot.value.idx = data.bot ? data.bot.idx : null;
        bot.value.uart = data.bot ? data.bot.uart : null;
      }
    };

    session.socket.onerror = event => {
      crash.value = 'lost connection to the server';
    };
  }

  function togglePause() {
    paused.value = !paused.value;

    if (paused.value) {
      session.socket.close();
    } else {
      join(
        localStorage.getItem('worldId'),
        localStorage.getItem('botId'),
      );
    }
  }

  function handleWorldChange(worldId) {
    join(worldId, null);
  }

  async function handleBotUpload(file) {
    if (session == null || session.socket == null) {
      return;
    }

    var response = await fetch(
      `${import.meta.env.VITE_HTTP_URL}/worlds/${session.worldId}/bots`,
      {
        method: 'POST',
        body: file,
      },
    );

    var response = await response.json();

    join(session.worldId, response.id);
  }

  function handleBotDisconnect() {
    if (session == null) {
      return;
    }

    join(session.worldId, null);
  }

  function handleBotFollow(follow) {
    if (session == null || bot.value == null) {
      return;
    }

    bot.value.is_followed = follow;
  }

  function handleBotClick(id) {
    if (session == null) {
      return;
    }

    if (bot.value != null && bot.value.id == id && !paused.value) {
      join(session.worldId, null);
    } else {
      join(session.worldId, id);
    }
  }

  // ---

  const prevWorldId = localStorage.getItem('worldId');
  const prevBotId = localStorage.getItem('botId');

  if (prevWorldId) {
    join(prevWorldId, prevBotId);
  }
</script>

<template>
  <template v-if="crash == null">
    <Nav
      :session="session"
      :paused="paused"
      @world-change="handleWorldChange"
      @pause="togglePause" />

    <main>
      <Canvas
        :map="map"
        :bot="bot"
        :bots="bots"
        :camera="camera"
        :paused="paused" />

      <Side
        :mode="mode"
        :bot="bot"
        :bots="bots"
        :paused="paused"
        @bot-upload="handleBotUpload"
        @bot-disconnect="handleBotDisconnect"
        @bot-follow="handleBotFollow"
        @bot-click="handleBotClick"/>
    </main>
  </template>

  <template v-else>
    <p style="margin: 0">
      whoopsie, the game has ✨ crashed ✨
    </p>

    <p>
      {{ crash }}
    </p>

    <p style="margin-top: 0">
      please refresh the page to restart
    </p>
  </template>
</template>

<style scoped>
  main {
    display: flex;
    align-items: stretch;
    flex-grow: 1;
  }
</style>
