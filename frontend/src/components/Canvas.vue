<script setup>
  import { ref, onMounted, watch } from 'vue';

  const emit = defineEmits(['resize']);
  const props = defineProps(['map', 'bot', 'bots', 'paused']);
  const canvas = ref(null);
  const canvasWrapper = ref(null);

  let ctxt = null;
  let scale = 1.0;
  let textScale = 1.0;
  let textMetrics = null;
  let chars = { x: 0, y: 0 };

  watch(() => [props.map, props.paused], _ => {
    refresh();
  });

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

  function resize() {
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

    emit('resize', chars);
  }

  function refresh() {
    const { map, bot, bots, paused } = props;

    if (ctxt == null) {
      return;
    }

    ctxt.clearRect(0, 0, canvas.value.width, canvas.value.height);

    if (map == null) {
      ctxt.fillStyle = 'rgb(0, 255, 128)';
      ctxt.fillText('connecting...', 0, textMetrics.height);

      return;
    }

    let nth = 0;

    for (let y = 0; y <= chars.y; y += 1) {
      for (let x = 0; x <= chars.x; x += 1) {
        const tile = map[nth];
        const tileBase = String.fromCharCode(tile >> 24);
        const tileMeta0 = (tile >> 16) & 0xff;

        let tileColor = 'rgb(80, 80, 80)';
        let tileOffsetY = 0;

        switch (tileBase) {
          case '.':
            tileOffsetY = -0.45;
            break;

          case '=':
            tileColor = 'rgb(255, 106, 128)';
            tileOffsetY = -0.15;
            break;

          case '@':
            const tileBotIdx = tileMeta0;

            if (bot && bot.idx == tileBotIdx) {
              tileColor = 'rgb(0, 128, 255)';
            } else {
              tileColor = 'rgb(0, 255, 128)';
            }

            tileOffsetY = -0.15;
            break;
        }

        if (paused) {
          tileColor = 'rgb(40, 40, 40)';
        }

        ctxt.fillStyle = tileColor;

        ctxt.fillText(
          tileBase,
          textMetrics.width * x,
          textMetrics.height * (y + tileOffsetY),
        );

        nth += 1;
      }
    }
  }
</script>

<template>
  <div ref="canvasWrapper">
    <canvas ref="canvas" />
  </div>
</template>

<style scoped>
  div {
    position: relative;
    border: 1px dashed #444444;
    flex-grow: 1;
  }

  canvas {
    position: absolute;
  }
</style>
