<script setup lang="ts">
import { ref, onMounted, watch, toRaw } from "vue";
import { botIdToColor } from "@/utils/bot";
import type { GameWorld } from "./World";

const emit = defineEmits<{
  botJoin: [string];
}>();

const props = defineProps<{
  world: GameWorld;
  paused: boolean;
}>();

const canvas = ref(null);
const canvasWrapper = ref(null);
const pausedColorA = "rgb(40, 40, 40)";
const pausedColorB = "rgb(80, 80, 80)";

let ctxt: CanvasRenderingContext2D = null;

// Size of a single tile, in pixels
let tileSize = {
  w: 0,
  h: 0,
};

// Size of the entire canvas, in tiles
let canvasSize = {
  x: 0,
  y: 0,
};

// Camera offset, in tiles
let offset = {
  x: 0,
  y: 0,
};

let mouse = {
  // Position, in pixels relative to the canvas
  pos: {
    x: 0,
    y: 0,
  },

  // Id of bot the mouse is hovering over, if any
  hoveringOver: null,
};

function resize(): void {
  if (canvasWrapper.value == null) {
    // Can happen during development if Vue decides it's time to reload the
    // component
    return;
  }

  const width = canvasWrapper.value.clientWidth;
  const height = canvasWrapper.value.clientHeight;
  const dpr = window.devicePixelRatio || 1;

  canvas.value.width = width;
  canvas.value.height = height;

  ctxt.scale(dpr, dpr);
  ctxt.font = `30px Sono`;

  const ch = ctxt.measureText("@");

  tileSize = {
    w: ch.width,
    h: ch.actualBoundingBoxAscent + ch.actualBoundingBoxDescent + 2.0,
  };

  canvasSize = {
    x: Math.round(width / dpr / tileSize.w),
    y: Math.round(height / dpr / tileSize.h),
  };
}

function update(): void {
  if (canvasWrapper.value == null) {
    // Can happen during development if Vue decides it's time to reload the
    // component
    return;
  }

  const map = toRaw(props.world.map.value);
  const camera = toRaw(props.world.camera.value);

  // ---

  offset = {
    x: camera.x - Math.round(canvasSize.x / 2),
    y: camera.y - Math.round(canvasSize.y / 2),
  };

  // ---

  mouse.hoveringOver = null;

  if (mouse) {
    const mouseTilePos = {
      x: Math.floor(offset.x + mouse.pos.x / tileSize.w),
      y: Math.floor(offset.y + mouse.pos.y / tileSize.h),
    };

    if (mouseTilePos.x >= 0 && mouseTilePos.y >= 0) {
      const mouseTileIdx = mouseTilePos.y * map.size[0] + mouseTilePos.x;
      const mouseBot = map.bots[mouseTileIdx];

      if (mouseBot) {
        mouse.hoveringOver = mouseBot.id;
      }
    }
  }

  document.body.style.cursor = mouse.hoveringOver ? "pointer" : "default";
}

function draw(): void {
  if (canvasWrapper.value == null) {
    // Can happen during development if Vue decides it's time to reload the
    // component
    return;
  }

  const status = props.world.status.value;
  const camera = props.world.camera.value;
  const map = props.world.map.value;

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
      } else {
        // Should be unreachable, but let's keep it just in case - there might
        // happen that it takes one frame more for the map and camera data to
        // arrive etc.
      }

      break;
  }
}

function drawStatus(): void {
  const status = props.world.status.value;
  const x = 8;
  const y = tileSize.h + 8;

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
  const paused = toRaw(props.paused);

  for (let y = 0; y <= canvasSize.y; y += 1) {
    for (let x = 0; x <= canvasSize.x; x += 1) {
      const tilePos = { x: offset.x + x, y: offset.y + y };

      if (
        tilePos.x < 0 ||
        tilePos.y < 0 ||
        tilePos.x >= map.size[0] ||
        tilePos.y >= map.size[1]
      ) {
        continue;
      }

      const tileIdx = tilePos.y * map.size[0] + tilePos.x;
      const tile = map.tiles[tileIdx] ?? 0;
      const tileBot = map.bots[tileIdx] ?? null;

      let tileFg: string;
      let tileBg: string = null;
      let tileChar: string;
      let tileOffset = { x: 0.0, y: 0.0 };

      if (tileBot) {
        tileChar = "@";
        tileOffset = { x: -0.025, y: -0.125 };

        if (tileBot.known) {
          if (paused) {
            tileBg = pausedColorB;
            tileFg = "#000000";
          } else {
            tileBg = botIdToColor(tileBot.id, "bg");
            tileFg = "#ffffff";
          }
        } else {
          if (paused) {
            tileFg = pausedColorB;
          } else {
            tileFg = botIdToColor(tileBot.id);
          }
        }
      } else {
        tileFg = paused ? pausedColorA : "rgb(80, 80, 80)";
        tileChar = String.fromCharCode(tile >> 24);

        switch (tileChar) {
          case ".":
            tileOffset = { x: 0.035, y: -0.4 };
            break;
        }
      }

      const tilePixelPos = {
        x: tileSize.w * (x + tileOffset.x),
        y: tileSize.h * (y + tileOffset.y + 1),
      };

      if (tileBot && mouse.hoveringOver && tileBot.id == mouse.hoveringOver) {
        tileBg = "#ffffff";
        tileFg = "#000000";
      }

      if (tileBg) {
        ctxt.fillStyle = tileBg;

        ctxt.fillRect(
          tilePixelPos.x,
          tilePixelPos.y - tileSize.h * 0.9,
          tileSize.w,
          tileSize.h,
        );
      }

      ctxt.fillStyle = tileFg;
      ctxt.fillText(tileChar, tilePixelPos.x, tilePixelPos.y);
    }
  }
}

function drawCarets(blink: boolean): void {
  const bot = toRaw(props.world.bot.value);
  const bots = toRaw(props.world.bots.value);
  const paused = toRaw(props.paused);
  const selectedBotId = bot?.id;

  ctxt.save();
  ctxt.translate(-offset.x * tileSize.w, -offset.y * tileSize.h);

  for (const [botId, bot] of Object.entries(bots)) {
    let botColor = paused ? pausedColorB : botIdToColor(botId);

    if (botId == selectedBotId && blink && !paused) {
      botColor = "#ffffff";
    }

    if (botId == mouse.hoveringOver) {
      botColor = "#ffffff";
    }

    ctxt.save();
    ctxt.translate(tileSize.w * bot.pos[0], tileSize.h * (bot.pos[1] + 1));
    ctxt.translate(tileSize.w / 2, -tileSize.h / 2);

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
      d1 = tileSize.h;
      d2 = tileSize.w;
    } else {
      d1 = tileSize.w;
      d2 = tileSize.h;
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

function handleMouseMove(ev: any): void {
  const dpr = window.devicePixelRatio || 1;

  mouse.pos = {
    x: ev.layerX / dpr,
    y: ev.layerY / dpr,
  };

  update();
  draw();
}

function handleClick(): void {
  if (mouse.hoveringOver) {
    emit("botJoin", mouse.hoveringOver);
  }
}

// ---

watch([props.world.map, props.world.bots], () => {
  update();
  draw();
});

watch(
  [props.world.bot, props.world.camera],
  () => {
    update();
    draw();
  },
  { deep: true },
);

watch(
  () => props.paused,
  () => {
    update();
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
    <canvas
      ref="canvas"
      @mousemove.prevent="handleMouseMove"
      @click.prevent="handleClick"
    />
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
