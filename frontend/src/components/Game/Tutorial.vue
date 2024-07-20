<script setup lang="ts">
import type { GameController } from "./Controller";

const emit = defineEmits<{
  next: [];
}>();

defineProps<{
  ctrl: GameController;
}>();
</script>

<script lang="ts">
export async function setup(ctrl: GameController): Promise<void> {
  ctrl.disableButton("help");
  ctrl.disableButton("pause");
  ctrl.disableButton("connectToBot");
  ctrl.disableButton("uploadBot");

  await ctrl.waitFor("server.ready");

  // ---

  ctrl.openHelp(1);

  await ctrl.waitFor("help.continue");

  // ---

  ctrl.openHelp(2);
  ctrl.enableButton("uploadBot");
  ctrl.highlightButton("uploadBot");

  await ctrl.waitFor("bot.uploaded");
}
</script>

<template>
  <dialog class="game-tutorial" :open="ctrl.helpId.value != null">
    <nav>
      <div class="dialog-title">tutorial</div>
    </nav>

    <template v-if="ctrl.helpId.value == 1">
      <p>hey there soldier, welcome to the tutorial 🫡🫡🫡</p>
      <p>start by following this carefully crafted instruction:</p>

      <pre>
      $ git clone https://github.com/patryk27/kartoffel
      $ cd kartoffel</pre
      >

      <p>
        ... and then press
        <button class="highlighted" @click="ctrl.emit('help.continue')">
          continue
        </button>
        once you're set up.
      </p>
    </template>

    <template v-else-if="ctrl.helpId.value == 2">
      <p>TODO</p>
    </template>
  </dialog>
</template>

<style scoped>
.game-tutorial {
  max-width: 768px;

  p {
    &:last-child {
      margin-bottom: 0;
    }
  }
}
</style>
