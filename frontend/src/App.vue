<script setup>
  import { ref, onMounted } from 'vue';
  import Canvas from './components/Canvas.vue';
  import Nav from './components/Nav.vue';
  import Side from './components/Side.vue';

  const map = ref(null);
  const mode = ref(null);
  const bot = ref(null);
  const bots = ref(null);
  const paused = ref(false);

  let world = null;
  let chars = { x: 1, y: 1 };

  document.onkeydown = () => {
    if (world == null || world.socket == null || paused.value) {
      return;
    }

    if (bot != null && bot.value.is_followed) {
      followBot(false);
    }

    let worldEvent = null;

    switch (event.keyCode) {
      case 37: // left
      case 65: // a
        worldEvent = { op: 'move-camera', args: { dx: -8 } };
        break;

      case 38: // up
      case 87: // w
        worldEvent = { op: 'move-camera', args: { dy: -8 } };
        break;

      case 39: // right
      case 68: // d
        worldEvent = { op: 'move-camera', args: { dx: 8 } };
        break;

      case 40: // down
      case 83: // s
        worldEvent = { op: 'move-camera', args: { dy: 8 } };
        break;
    }

    if (worldEvent != null) {
      world.socket.send(JSON.stringify(worldEvent));
    }
  };

  function resize(newChars) {
    chars = newChars;

    if (world != null && world.socket != null) {
      world.socket.send(JSON.stringify({
        op: 'scale-camera',
        args: chars,
      }));
    }
  }

  function pause() {
    paused.value = !paused.value;
  }

  function changeWorld(newWorldId) {
    if (world != null) {
      if (world.id == newWorldId) {
        return;
      }

      if (world.socket != null) {
        world.socket.close();
      }
    }

    localStorage.removeItem('botId');
    localStorage.removeItem('worldId');

    world = {
      id: newWorldId,
      socket: new WebSocket(
        `${import.meta.env.VITE_WS_URL}/worlds/${newWorldId}`
      ),
    };

    return new Promise((resolve, reject) => {
      paused.value = false;

      world.socket.onopen = () => {
        localStorage.setItem('worldId', world.id);

        world.socket.send(JSON.stringify({
          op: 'scale-camera',
          args: chars,
        }));

        resolve();
      };

      world.socket.onmessage = event => {
        if (paused.value) {
          return;
        }

        const data = JSON.parse(event.data);

        map.value = data.map;
        mode.value = data.mode;
        bots.value = data.bots;

        const botIdToIdx = id => {
          for (let idx = 0; idx < data.bots.length; idx += 1) {
            if (data.bots[idx].id == id) {
              return idx;
            }
          }

          return -1;
        };

        if (bot.value) {
          bot.value.idx = botIdToIdx(bot.value.id);
          bot.value.uart = data.bot ? data.bot.uart : '';
        }
      };
    });
  }

  async function uploadBot(file) {
    if (world == null || world.socket == null || paused.value) {
      return;
    }

    var response = await fetch(
      `${import.meta.env.VITE_HTTP_URL}/worlds/${world.id}/bots`,
      {
        method: 'POST',
        body: file,
      },
    );

    var response = await response.json();

    connectToBot(response.id);
  }

  function disconnectFromBot() {
    if (world == null || world.socket == null) {
      return;
    }

    bot.value = null;

    world.socket.send(JSON.stringify({
      op: 'disconnect-from-bot',
    }));

    localStorage.removeItem('botId');
  }

  function connectToBot(id) {
    if (world == null || world.socket == null) {
      return;
    }

    bot.value = {
      id,
      is_followed: true,
    };

    world.socket.send(JSON.stringify({
      op: 'connect-to-bot',
      args: { id },
    }));

    world.socket.send(JSON.stringify({
      op: 'follow-bot',
    }));

    localStorage.setItem('botId', bot.value.id);

    paused.value = false;
  }

  function followBot(follow) {
    if (world == null || world.socket == null) {
      return;
    }

    world.socket.send(JSON.stringify({
      op: follow ? 'follow-bot' : 'unfollow-bot',
    }));

    bot.value.is_followed = follow;
  }

  function changeBot(id) {
    if (bot.value && bot.value.id == id && !paused.value) {
      disconnectFromBot();
    } else {
      connectToBot(id);
    }
  }

  // ---

  const savedWorldId = localStorage.getItem('worldId');
  const savedBotId = localStorage.getItem('botId');

  if (savedWorldId) {
    changeWorld(savedWorldId).then(() => {
      if (savedBotId) {
        changeBot(savedBotId);
      }
    });
  }
</script>

<template>
  <Nav
    :world="world"
    :paused="paused"
    @world-change="changeWorld"
    @pause="pause" />

  <main>
    <Canvas
      :map="map"
      :bot="bot"
      :bots="bots"
      :paused="paused"
      @resize="resize" />

    <Side
      :mode="mode"
      :bot="bot"
      :bots="bots"
      :paused="paused"
      @bot-upload="uploadBot"
      @bot-disconnect="disconnectFromBot"
      @bot-follow="followBot"
      @bot-click="changeBot"/>
  </main>
</template>

<style scoped>
  main {
    display: flex;
    align-items: stretch;
    flex-grow: 1;
  }
</style>
