<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.onSlide(6, () => {
  ctrl.enableButton("uploadBot");
  ctrl.highlightButton("uploadBot");

  // TODO wait just once
  ctrl.waitFor("join").then(() => {
    ctrl.openSlide(7);
  });
});
</script>

<template>
  <p>
    cool! -- now click the <kbd>[upload bot]</kbd> button and choose
    <kbd>./kartoffel</kbd>
  </p>

  <p>
    it should be located in the main directory, next to
    <kbd>./README.md</kbd>, <kbd>./build</kbd> etc.
  </p>

  <p>
    (( fwiw, this file is an
    <a
      target="_blank"
      href="https://en.wikipedia.org/wiki/Executable_and_Linkable_Format"
      >*.elf binary</a
    >
    that contains
    <a target="_blank" href="https://en.wikipedia.org/wiki/RISC-V">
      risc-v opcodes
    </a>
    encoding our source code from the previous slide ))
  </p>

  <footer>
    <p>
      if something went wrong, feel free to
      <a href="#" @click="ctrl.openSlide(5)">check out the previous slide</a>
    </p>
  </footer>
</template>
