<script setup lang="ts">
import { ref, onMounted, watch, toRaw } from "vue";
import { botIdToColor } from "@/utils/bot";
import type { GameWorld } from "./World";

const props = defineProps<{
  world: GameWorld;
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

function resize(): void {
  if (canvasWrapper.value == null) {
    return;
  }

  const width = canvasWrapper.value.clientWidth;
  const height = canvasWrapper.value.clientHeight;
  const dpr = window.devicePixelRatio || 1;

  scale = 13.5;
  textScale = scale * 2.0;
  canvas.value.width = width;
  canvas.value.height = height;

  ctxt.scale(dpr, dpr);
  ctxt.font = `${textScale}px Sono`;

  charMetrics = ctxt.measureText("@");

  charMetrics.height =
    charMetrics.actualBoundingBoxAscent +
    charMetrics.actualBoundingBoxDescent +
    2.0;

  chars = {
    x: Math.round(width / dpr / charMetrics.width),
    y: Math.round(height / dpr / charMetrics.height),
  };
}

function draw(): void {
  const status = props.world.status.value;
  const camera = props.world.camera.value;
  const map = props.world.map.value;

  if (ctxt == null || canvas.value == null) {
    return;
  }

  ctxt.clearRect(0, 0, canvas.value.width, canvas.value.height);

  switch (status) {
    case "connecting":
    case "reconnecting":
      drawStatus();
      break;

    case "connected":
      if (map && camera) {
        const blink = Date.now() % 1000 <= 500;

        drawTiles();
        drawCarets(blink);
      }

      break;
  }
}

function drawStatus(): void {
  const status = props.world.status.value;
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

function drawTiles(): void {
  const map = toRaw(props.world.map.value);
  const camera = toRaw(props.world.camera.value);
  const paused = toRaw(props.paused);
  const cw = charMetrics.width;
  const ch = charMetrics.height;

  for (let y = 0; y <= chars.y; y += 1) {
    for (let x = 0; x <= chars.x; x += 1) {
      const tileX = camera.x - Math.round(chars.x / 2) + x;
      const tileY = camera.y - Math.round(chars.y / 2) + y;

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

      let tileFg: string;
      let tileBg: string = null;
      let tileChar: string;
      let tileOffsetY = 0.0;
      let tileOffsetX = 0.0;

      if (tileBot) {
        tileChar = "@";
        tileOffsetX = -0.025;
        tileOffsetY = -0.125;

        if (tileBot.known) {
          if (paused) {
            tileBg = pausedColor;
            tileFg = "#000000";
          } else {
            tileBg = botIdToColor(tileBot.id, "bg");
            tileFg = "#ffffff";
          }
        } else {
          if (paused) {
            tileFg = pausedColor;
          } else {
            tileFg = botIdToColor(tileBot.id);
          }
        }
      } else {
        tileFg = paused ? pausedColor : "rgb(80, 80, 80)";
        tileChar = String.fromCharCode(tile >> 24);

        switch (tileChar) {
          case ".":
            tileOffsetX = 0.035;
            tileOffsetY = -0.4;
            break;
        }
      }

      const tx = cw * (x + tileOffsetX);
      const ty = ch * (y + tileOffsetY + 1);

      if (tileBg) {
        ctxt.fillStyle = tileBg;
        ctxt.fillRect(tx, ty - ch * 0.9, cw, ch);
      }

      ctxt.fillStyle = tileFg;
      ctxt.fillText(tileChar, tx, ty);
    }
  }
}

function drawCarets(blink: boolean): void {
  const bot = toRaw(props.world.bot.value);
  const bots = toRaw(props.world.bots.value);
  const camera = toRaw(props.world.camera.value);
  const paused = toRaw(props.paused);
  const cw = charMetrics.width;
  const ch = charMetrics.height;
  const selectedBotId = bot?.id;

  ctxt.save();

  ctxt.translate(
    (Math.round(chars.x / 2) - camera.x) * cw,
    (Math.round(chars.y / 2) - camera.y) * ch,
  );

  for (const [botId, bot] of Object.entries(bots)) {
    let botColor = paused ? pausedColor : botIdToColor(botId);

    if (botId == selectedBotId && blink && !paused) {
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

    let d1: number;
    let d2: number;

    if (bot.dir == "<" || bot.dir == ">") {
      d1 = ch;
      d2 = cw;
    } else {
      d1 = cw;
      d2 = ch;
    }

    ctxt.translate(0, -d2 * 1.025);
    ctxt.beginPath();
    ctxt.moveTo(-0.4 * d1, 0.4 * d2);
    ctxt.lineTo(0, 0);
    ctxt.lineTo(0.4 * d1, 0.4 * d2);

    ctxt.strokeStyle = botColor;
    ctxt.lineWidth = 2;
    ctxt.stroke();

    ctxt.restore();
  }

  ctxt.restore();
}

// ---

watch([props.world.map, props.world.bots], () => {
  draw();
});

watch(
  [props.world.bot, props.world.camera],
  () => {
    draw();
  },
  { deep: true },
);

watch(
  () => props.paused,
  () => {
    draw();
  },
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
  border: 1px solid var(--gray);
  flex-grow: 1;
  overflow: hidden;

  canvas {
    display: block;
    position: absolute;
  }
}
</style>
