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

    await ctrl.waitFor("tutorial.continue");
  } else {
    ctrl.openHelp(4);

    await ctrl.waitFor("tutorial.continue");

    ctrl.openHelp(5);

    await ctrl.waitFor("tutorial.continue");

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
        once you're ready.
      </p>

      <footer>
        <p>
          <b>cool bear's hot tip:</b> you can leave the tutorial at any time by
          using the <kbd>[leave]</kbd> button in the top left corner
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

        <p>or</p>

        <li>
          if you're more into step-by-step tutorials,

          <button @click="ctrl.emit('tutorial.routeB')" class="highlighted">
            click here
          </button>
        </li>
      </ul>
    </template>

    <template v-else-if="ctrl.helpId.value == 3">
      <p>in this case, here's a quick introduction:</p>

      <ul>
        <li>
          see <kbd>README.md</kbd> in the repository for building instructions
        </li>

        <li>use the <kbd>[upload bot]</kbd> button to upload the binary</li>
        <li>navigate map using W/A/S/D or arrow keys</li>
        <li>bots are represented with the @ char</li>

        <li>
          bots uploaded by you will have
          <span class="inverted">inverted colors</span>
        </li>

        <li>
          use your ide's <kbd>go to definition</kbd> feature to discover how
          functions (such as <kbd>radar_scan()</kbd>) work
        </li>
      </ul>

      <p>
        when you press the button below, you'll be redirected back to the home
        page - from there, just choose any world and start playing
      </p>

      <p style="text-align: center">good luck and have fun!</p>

      <p style="text-align: right">
        <button @click="ctrl.emit('tutorial.continue')" class="highlighted">
          i'm ready
        </button>
      </p>
    </template>

    <template v-else-if="ctrl.helpId.value == 4">
      <p>
        in this case, for the purposes of this tutorial, we'll be starting from
        scratch - open <kbd>src/main.rs</kbd> and replace the code there with:
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

      <p>this doesn't do anything yet, but baby steps</p>

      <p>
        first, as compared to usual rust code, implementing a firmware is a bit
        funky - what's most important is that there's no standard library
        (<kbd>#![no_std]</kbd>), so you can't access
        <kbd>std::fs</kbd> etc.
      </p>

      <p>
        that's because the game simulates an actual small robot, with limited
        CPU and RAM, and without access to the external world (besides specific
        emulated peripherals such as the radar)
      </p>

      <p>
        you <i>can</i> use all language constructs, though - functions, traits
        etc. all work; you can even allocate, just remember to import
        <kbd>Vec</kbd> from the <kbd>alloc</kbd> crate instead of <kbd>std</kbd>
      </p>

      <p style="text-align: right">
        <button @click="ctrl.emit('tutorial.continue')" class="highlighted">
          got it
        </button>
      </p>
    </template>

    <template v-else-if="ctrl.helpId.value == 5">
      <p>alright, with theory in mind let's make the robot do something now</p>
    </template>
  </dialog>
</template>

<style scoped>
.game-tutorial {
  width: 768px;

  p,
  ul {
    &:last-child {
      margin-bottom: 0;
    }
  }

  pre {
    margin-left: 4ch;
  }

  .inverted {
    background: white;
    color: black;
  }
}
</style>
