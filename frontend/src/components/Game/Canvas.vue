<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { botIdToColor } from "@/utils/bot";
import type {
  GameBot,
  GameBots,
  GameCamera,
  GameMap,
  GameStatus,
} from "../Game.vue";

const props = defineProps<{
  map?: GameMap;
  bot?: GameBot;
  bots?: GameBots;
  camera?: GameCamera;
  status: GameStatus;
  paused: boolean;
}>();

const canvas = ref(null);
const canvasWrapper = ref(null);
const pausedColor = "rgb(40, 40, 40)";

let ctxt = null;
let scale = 1.0;
let textScale = 1.0;
let charMetrics = null;
let chars = { x: 0, y: 0 };

function resize() {
  if (canvasWrapper.value == null) {
    return;
  }

  const width = canvasWrapper.value.clientWidth;
  const height = canvasWrapper.value.clientHeight;

  scale = 16.0; // TODO consider making dynamic
  textScale = scale * 2.0;
  canvas.value.width = width;
  canvas.value.height = height;

  ctxt.scale(window.devicePixelRatio || 1, window.devicePixelRatio || 1);
  ctxt.font = `${textScale}px Sono`;

  charMetrics = ctxt.measureText("@");

  charMetrics.height =
    charMetrics.actualBoundingBoxAscent +
    charMetrics.actualBoundingBoxDescent +
    2.0;

  chars = {
    x: Math.round(width / charMetrics.width),
    y: Math.round(height / charMetrics.height),
  };
}

function draw(): void {
  const { camera, map, status } = props;

  if (ctxt == null || canvas.value == null) {
    return;
  }

  ctxt.clearRect(0, 0, canvas.value.width, canvas.value.height);

  const isBlinking = Date.now() % 1000 <= 500;

  switch (status) {
    case "connecting":
    case "reconnecting":
      drawStatus();
      break;

    case "connected":
      if (map == null || camera == null) {
        break;
      }

      drawTiles(isBlinking);
      drawBots(isBlinking);
      break;
  }
}

function drawStatus(): void {
  const { status } = props;
  const x = 8;
  const y = charMetrics.height + 8;

  switch (status) {
    case "connecting":
      ctxt.fillStyle = "rgb(0, 255, 128)";
      ctxt.fillText("connecting...", x, y);
      break;

    case "reconnecting":
      ctxt.fillStyle = "rgb(0, 255, 128)";
      ctxt.fillText("connection lost, reconnecting...", x, y);
      break;
  }
}

function drawTiles(isBlinking: boolean): void {
  const { map, bot, camera, paused } = props;
  const cw = charMetrics.width;
  const ch = charMetrics.height;

  for (let y = 0; y <= chars.y; y += 1) {
    for (let x = 0; x <= chars.x; x += 1) {
      const tileX = camera.x - Math.round(chars.x / 2) + x + 1;
      const tileY = camera.y - Math.round(chars.y / 2) + y + 1;

      if (
        tileX < 0 ||
        tileY < 0 ||
        tileX >= map.size[0] ||
        tileY >= map.size[1]
      ) {
        continue;
      }

      const tileIdx = tileY * map.size[0] + tileX;
      const tile = map.tiles[tileIdx] ?? 0;
      const tileBot = map.bots[tileIdx] ?? null;

      let tileChar: string;
      let tileColor: string;
      let tileOffsetY: number;

      if (tileBot) {
        tileChar = "@";
        tileColor = botIdToColor(tileBot.id);
        tileOffsetY = -0.1;

        if (tileBot.id == bot?.id && isBlinking) {
          tileColor = "#ffffff";
        }
      } else {
        tileChar = String.fromCharCode(tile >> 24);
        tileColor = "rgb(80, 80, 80)";
        tileOffsetY = -0.0;

        switch (tileChar) {
          case ".":
            tileOffsetY = -0.45;
            break;

          case "=":
            tileColor = "rgb(255, 106, 128)";
            tileOffsetY = -0.15;
            break;
        }
      }

      ctxt.fillStyle = paused ? pausedColor : tileColor;
      ctxt.fillText(tileChar, cw * x, ch * (y + tileOffsetY + 1));
    }
  }
}

function drawBots(isBlinking: boolean): void {
  const { bot, bots, camera, paused } = props;
  const cw = charMetrics.width;
  const ch = charMetrics.height;
  const selectedBotId = bot?.id;

  ctxt.save();

  ctxt.translate(
    (Math.round(chars.x / 2) - camera.x - 1) * cw,
    (Math.round(chars.y / 2) - camera.y - 1) * ch,
  );

  for (const [botId, bot] of Object.entries(bots)) {
    let botColor = paused ? pausedColor : botIdToColor(botId);

    if (botId == selectedBotId && isBlinking) {
      botColor = "#ffffff";
    }

    ctxt.save();
    ctxt.translate(cw * bot.pos[0], ch * (bot.pos[1] + 1));
    ctxt.translate(cw / 2, -ch / 2);

    switch (bot.dir) {
      case "<":
        ctxt.rotate(-Math.PI / 2);
        break;

      case ">":
        ctxt.rotate(Math.PI / 2);
        break;

      case "v":
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

watch(
  () => [props.map, props.bots, props.status, props.paused],
  (_) => {
    draw();
  },
);

watch(
  () => [props.bot, props.camera],
  (_) => {
    draw();
  },
  { deep: true },
);

onMounted(() => {
  ctxt = canvas.value.getContext("2d");

  const observer = new ResizeObserver(() => {
    resize();
    draw();
  });

  observer.observe(canvasWrapper.value);

  document.fonts.ready.then(() => {
    resize();
    draw();
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
  overflow: hidden;

  canvas {
    display: block;
    position: absolute;
  }
}
</style>
