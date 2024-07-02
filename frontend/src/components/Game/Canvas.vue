<script setup>
  import { ref, onMounted, watch } from 'vue';
  import { botIdToColor } from '@/utils/bot.ts';

  const props = defineProps([
    'map',
    'bot',
    'bots',
    'camera',
    'status',
    'paused',
  ]);

  const canvas = ref(null);
  const canvasWrapper = ref(null);
  const pausedColor = 'rgb(40, 40, 40)';

  let ctxt = null;
  let scale = 1.0;
  let textScale = 1.0;
  let charMetrics = null;
  let chars = { x: 0, y: 0 };

  function resize() {
    if (canvasWrapper.value == null) {
      return;
    }

    scale = Math.max(
      Math.min(
        canvasWrapper.value.clientWidth,
        canvasWrapper.value.clientHeight
      ) / 64.0,
      14.0
    );

    textScale = scale * 2.0;
    canvas.value.width = canvasWrapper.value.clientWidth;
    canvas.value.height = canvasWrapper.value.clientHeight;
    ctxt.font = `${textScale}px Sono`;

    charMetrics = ctxt.measureText('@');

    charMetrics.height =
      charMetrics.actualBoundingBoxAscent
      + charMetrics.actualBoundingBoxDescent
      + 2.0;

    chars = {
      x: Math.round(canvasWrapper.value.clientWidth / charMetrics.width),
      y: Math.round(canvasWrapper.value.clientHeight / charMetrics.height),
    };
  }

  function draw() {
    const { camera, map, status } = props;

    if (ctxt == null || canvas.value == null) {
      return;
    }

    ctxt.clearRect(0, 0, canvas.value.width, canvas.value.height);

    switch (status) {
      case 'connecting':
      case 'reconnecting':
        drawStatus();
        break;

      case 'connected':
        if (map == null || camera == null) {
          break;
        }

        drawTiles();
        drawBots();
        break;
    }
  }

  function drawStatus() {
    const { status } = props;
    const x = 8;
    const y = charMetrics.height + 8;

    switch (status) {
      case 'connecting':
        ctxt.fillStyle = 'rgb(0, 255, 128)';
        ctxt.fillText('connecting...', x, y);
        break;

      case 'reconnecting':
        ctxt.fillStyle = 'rgb(0, 255, 128)';
        ctxt.fillText('connection lost, reconnecting...', x, y);
        break;
    }
  }

  function drawTiles() {
    const { map, bot, bots, camera, paused } = props;
    const cw = charMetrics.width;
    const ch = charMetrics.height;

    for (let y = 0; y <= chars.y; y += 1) {
      for (let x = 0; x <= chars.x; x += 1) {
        const tileX = camera.x - Math.round(chars.x / 2) + x;
        const tileY = camera.y - Math.round(chars.y / 2) + y;

        if (tileX < 0 || tileY < 0 || tileX >= map.size[0] || tileY >= map.size[1]) {
          continue;
        }

        const tileIdx = tileY * map.size[0] + tileX;
        const tile = map.tiles[tileIdx] ?? 0;
        const tileBot = map.bots[tileIdx] ?? null;

        let tileChar;
        let tileColor;
        let tileOffsetY;

        if (tileBot) {
          tileChar = '@';
          tileColor = botIdToColor(tileBot.id);
          tileOffsetY = -0.1;

          if (tileBot.id == bot?.id) {
            tileChar = '&';
          }
        } else {
          tileChar = String.fromCharCode(tile >> 24);
          tileColor = 'rgb(80, 80, 80)';
          tileOffsetY = -0.0;

          switch (tileChar) {
            case '.':
              tileOffsetY = -0.45;
              break;

            case '=':
              tileColor = 'rgb(255, 106, 128)';
              tileOffsetY = -0.15;
              break;
          }
        }

        ctxt.fillStyle = paused ? pausedColor : tileColor;
        ctxt.fillText(tileChar, cw * x, ch * (y + tileOffsetY + 1));
      }
    }
  }

  function drawBots() {
    const { map, bot, bots, camera, paused } = props;
    const cw = charMetrics.width;
    const ch = charMetrics.height;

    ctxt.save();

    ctxt.translate(
      (Math.round(chars.x / 2) - camera.x) * cw,
      (Math.round(chars.y / 2) - camera.y) * ch,
    );

    for (const [botId, bot] of Object.entries(bots)) {
      const botColor = paused ? pausedColor : botIdToColor(botId);

      ctxt.save();
      ctxt.translate(cw * bot.pos[0], ch * (bot.pos[1] + 1));
      ctxt.translate(cw / 2, -ch / 2);

      switch (bot.dir) {
        case '<':
          ctxt.rotate(-Math.PI / 2);
          break;

        case '>':
          ctxt.rotate(Math.PI / 2);
          break;

        case 'v':
          ctxt.rotate(Math.PI);
          break;
      }

      ctxt.translate(0, -ch * 0.9);

      ctxt.beginPath();
      ctxt.moveTo(-0.4 * cw, 0.3 * ch);
      ctxt.lineTo(0, 0);
      ctxt.lineTo(0.4 * cw, 0.3 * ch);

      ctxt.strokeStyle = botColor;
      ctxt.lineWidth = 2;
      ctxt.stroke();

      ctxt.restore();
    }

    ctxt.restore();
  }

  // ---

  watch(() => [props.map, props.bots, props.status, props.paused], _ => {
    draw();
  });

  watch(() => [props.bot, props.camera], _ => {
    draw();
  }, { deep: true });

  onMounted(() => {
    document.fonts.ready.then(() => {
      ctxt = canvas.value.getContext('2d');

      ctxt.scale(
        window.devicePixelRatio || 1,
        window.devicePixelRatio || 1,
      );

      resize();
      draw();

      // TODO consider using resize observer
      setInterval(() => {
        resize();
        draw();
      }, 100);
    });
  });
</script>

<template>
  <div ref="canvasWrapper" class="game-canvas">
    <canvas ref="canvas" />
  </div>
</template>

<style scoped>
  .game-canvas {
    position: relative;
    border: 1px dashed #444444;
    flex-grow: 1;

    canvas {
      position: absolute;
    }
  }
</style>
