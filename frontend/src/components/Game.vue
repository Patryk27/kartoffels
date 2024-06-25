<script setup>
  import { ref, onMounted } from 'vue';
  import { storeSession } from '@/utils/session.ts';
  import Canvas from './Game/Canvas.vue';
  import Nav from './Game/Nav.vue';
  import Side from './Game/Side.vue';

  const emit = defineEmits(['leave', 'openTutorial']);
  const props = defineProps(['worldId', 'botId']);
  const map = ref(null);
  const mode = ref(null);
  const bot = ref(null);
  const bots = ref(null);
  const camera = ref(null);
  const paused = ref(false);

  let socket = null;

  function join(newBotId) {
    if (socket != null) {
      socket.close();
    }

    map.value = null;
    mode.value = null;
    bot.value = null;
    bots.value = null;
    camera.value = null;
    paused.value = false;

    // ---

    if (newBotId != null) {
      bot.value = {
        id: newBotId,
        is_followed: true,
      };
    }

    socket = new WebSocket(
      (newBotId == null)
        ? `${import.meta.env.VITE_WS_URL}/worlds/${props.worldId}`
        : `${import.meta.env.VITE_WS_URL}/worlds/${props.worldId}/bots/${newBotId}`
    );

    socket.onopen = () => {
      storeSession({
        worldId: props.worldId,
        botId: bot.value?.id,
      });
    };

    socket.onmessage = event => {
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
        bot.value.serial = data.bot ? data.bot.serial : null;
      }
    };

    socket.onerror = event => {
      if (newBotId == null) {
        window.onerror(`couldn't join world ${props.worldId}`);
      } else {
        alert(`couldn't join bot ${newBotId} - maybe it got killed?`);

        join(null);
      }
    };
  }

  function handleLeave() {
    emit('leave');
  }

  function handleOpenTutorial() {
    emit('openTutorial');
  }

  function handlePause() {
    paused.value = !paused.value;

    if (paused.value) {
      socket.close();
    } else {
      // TODO don't restart camera position
      join(bot.value?.id);
    }
  }

  async function handleBotUpload(file) {
    if (socket == null) {
      return;
    }

    try {
      var response = await fetch(
        `${import.meta.env.VITE_HTTP_URL}/worlds/${props.worldId}/bots`,
        {
          method: 'POST',
          body: file,
        },
      );

      if (response.status == 200) {
        var response = await response.json();

        join(response.id);
      } else {
        var response = await response.text();

        alert("err, your bot couldn't be uploaded:\n\n" + response);
      }
    } catch (err) {
      window.onerror(err);
    }
  }

  function handleBotConnect(id) {
    join(id);
  }

  function handleBotDisconnect() {
    join(null);
  }

  function handleBotClick(id) {
    if (bot.value != null && bot.value.id == id && !paused.value) {
      join(null);
    } else {
      join(id);
    }
  }

  // ---

  join(props.botId);

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
          handlePause();
          break;
      }
    };
  });
</script>

<template>
  <Nav
    :paused="paused"
    @leave="handleLeave"
    @pause="handlePause"
    @open-tutorial="handleOpenTutorial" />

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
      @bot-connect="handleBotConnect"
      @bot-disconnect="handleBotDisconnect"
      @bot-click="handleBotClick" />
  </main>
</template>

<style scoped>
  main {
    display: flex;
    align-items: stretch;
    flex-grow: 1;
  }
</style>
