<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.alterUi((ui) => {
  ui.enableDisconnectFromBot = false;
  ui.enableUploadBot = true;
  ui.highlightUploadBot = true;
});

ctrl.onOnce("server.bot-create", () => {
  ctrl.pause();
  ctrl.openTutorialSlide(7);
});
</script>

<template>
  <main>
    <p>
      cool! -- now click the <kbd>[upload bot]</kbd> button and choose
      <kbd>./kartoffel</kbd>
    </p>

    <p>
      this file should be located in the main directory, right next to
      <kbd>./README.md</kbd>, <kbd>./build</kbd> etc.
    </p>

    <p>
      (( it is an
      <a
        target="_blank"
        href="https://en.wikipedia.org/wiki/Executable_and_Linkable_Format"
        >*.elf binary</a
      >
      containing
      <a target="_blank" href="https://en.wikipedia.org/wiki/RISC-V">
        risc-v opcodes
      </a>
      that define how the bot should behave - it's a compiled version of the
      Rust code from before ))
    </p>
  </main>

  <footer>
    (need help?
    <a href="#" @click="ctrl.openTutorialSlide(5)">see the previous slide</a>)
  </footer>
</template>
