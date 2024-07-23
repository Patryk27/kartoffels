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

  ctrl.openHelp(1);

  await ctrl.waitFor("tutorial.continue");

  ctrl.openHelp(2);

  const routeA = ctrl.waitFor("tutorial.routeA").then((_) => "A");
  const routeB = ctrl.waitFor("tutorial.routeB").then((_) => "B");
  const route = await Promise.any([routeA, routeB]);

  if (route == "A") {
    ctrl.openHelp(3);
  } else {
    ctrl.openHelp(4);
    ctrl.enableButton("uploadBot");
    ctrl.highlightButton("uploadBot");

    await ctrl.waitFor("bot.uploaded");
  }
}
</script>

<template>
  <dialog class="game-tutorial" :open="ctrl.helpId.value != null">
    <nav>
      <div class="dialog-title">tutorial</div>
    </nav>

    <template v-if="ctrl.helpId.value == 1">
      <p>hey there soldier and welcome to the tutorial 🫡🫡🫡</p>

      <p>
        in here we'll go through the basics of programming the robots and
        navigating the game - this will take just a couple of minutes
      </p>

      <p>
        i'm assuming you know a bit of rust (calling functions, using ifs etc.,
        nothing too advanced)
      </p>

      <p>alrighty - start by following this carefully crafted instruction:</p>

      <pre>
$ git clone https://github.com/patryk27/kartoffel
$ cd kartoffel</pre
      >

      <p>
        ... and then press
        <button class="highlighted" @click="ctrl.emit('tutorial.continue')">
          continue
        </button>
        once you're set up.
      </p>

      <footer>
        <p>
          <b>cool bear's hot tip:</b>
          you can leave the tutorial at any time by using the
          <kbd>[leave]</kbd> button in the top left corner
        </p>
      </footer>
    </template>

    <template v-else-if="ctrl.helpId.value == 2">
      <p>
        cool! -- the repository you cloned contains a skeleton project with a
        simple robot already implemented
      </p>

      <p>now:</p>

      <ul>
        <li>
          if you're more into discovering things yourself and you'd like just a
          quick introduction to the interface,

          <button @click="ctrl.emit('tutorial.routeA')">click here</button>
        </li>

        <p>
          or
        </p>

        <li>
          if you're more into step-by-step tutorials,

          <button @click="ctrl.emit('tutorial.routeB')" class="highlighted">
            click here
          </button>
        </li>
      </ul>
    </template>

    <template v-else-if="ctrl.helpId.value == 3">
      <p>TODO</p>
    </template>

    <template v-else-if="ctrl.helpId.value == 4">
      <p>
        so, for the purposes of this tutorial, we'll be starting from scratch - open
        <kbd>src/main.rs</kbd> and replace the code with:
      </p>

      <pre>
#![no_std]
#![no_main]

use kartoffel::*;

#[no_mangle]
fn main() {
    //
}</pre
      >

      <p>
        as compared to usual rust code, implementing a firmware is a bit
        different - what's most important is that there's no standard library
        (<kbd>#![no_std]</kbd>), so you can't, say, access <kbd>std::fs</kbd>
      </p>

      <p>
        that's because the firmware will run inside a virtual machine of sorts -
      </p>
    </template>
  </dialog>
</template>

<style scoped>
.game-tutorial {
  width: 768px;

  p, ul {
    &:last-child {
      margin-bottom: 0;
    }
  }

  pre {
    margin-left: 4ch;
  }
}
</style>
