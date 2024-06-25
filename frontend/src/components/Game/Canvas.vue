<script setup>
  import { ref, onMounted, watch } from 'vue';
  import { botIdToColor } from '@/utils/bot.ts';

  const props = defineProps(['map', 'bot', 'bots', 'camera', 'paused']);
  const canvas = ref(null);
  const canvasWrapper = ref(null);

  let ctxt = null;
  let scale = 1.0;
  let textScale = 1.0;
  let textMetrics = null;
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

    textMetrics = ctxt.measureText('@');

    textMetrics.height =
      textMetrics.actualBoundingBoxAscent
      + textMetrics.actualBoundingBoxDescent
      + 2.0;

    chars = {
      x: Math.round(canvasWrapper.value.clientWidth / textMetrics.width),
      y: Math.round(canvasWrapper.value.clientHeight / textMetrics.height),
    };
  }

  function refresh() {
    const { map, bot, bots, camera, paused } = props;

    if (ctxt == null) {
      return;
    }

    ctxt.clearRect(0, 0, canvas.value.width, canvas.value.height);

    if (map == null || camera == null) {
      return;
    }

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
          tileOffsetY = -0.15;
        } else {
          tileChar = String.fromCharCode(tile >> 24);
          tileColor = 'rgb(80, 80, 80)';
          tileOffsetY = 0.0;

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

        if (paused) {
          tileColor = 'rgb(40, 40, 40)';
        }

        ctxt.fillStyle = tileColor;

        ctxt.fillText(
          tileChar,
          textMetrics.width * x,
          textMetrics.height * (y + tileOffsetY),
        );
      }
    }
  }

  // ---

  watch(() => [props.map, props.paused], _ => {
    refresh();
  });

  watch(() => [props.bot, props.bots, props.camera], _ => {
    refresh();
  }, { deep: true });

  onMounted(() => {
    document.fonts.ready.then(() => {
      ctxt = canvas.value.getContext('2d');

      ctxt.scale(
        window.devicePixelRatio || 1,
        window.devicePixelRatio || 1,
      );

      window.onresize = () => {
        resize();
      };

      resize();
      refresh();
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