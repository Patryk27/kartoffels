<script setup lang="ts">
import type { GameWorld } from "./World";

const emit = defineEmits<{
  close: [];
}>();

defineProps<{
  world: GameWorld;
}>();

function handleDisableSandboxHelp(): void {
  localStorage.setItem("popups.sandboxHelp", "disabled");
  emit("close");
}
</script>

<script lang="ts">
export function canOpenSandboxHelp(): boolean {
  return localStorage.getItem("popups.sandboxHelp") == null;
}
</script>

<template>
  <dialog class="game-help">
    <template v-if="world.id == 'sandbox'">
      <nav>
        <div class="dialog-title">help</div>

        <div class="dialog-buttons">
          <button @click="emit('close')">close</button>
        </div>
      </nav>

      <p>hey there soldier, welcome to the sandbox 🫡🫡🫡</p>

      <p>
        sandbox is a special, private world where you can experiment with your
        bots before uploading them online
      </p>

      <p>as compared to the online play, in here:</p>

      <ul>
        <li>
          you can use the <kbd>[configure]</kbd> button (located above) to
          change how the world looks like
        </li>

        <li>
          you can <kbd>[spawn roberto]</kbd>, the built-in moderately
          challenging bot
        </li>

        <li>you can destroy and kill bots</li>
      </ul>

      <p>
        not sure what all this means? no problem, i don't either! -- the
        eggheads left some extra instructions:
      </p>

      <pre>
      $ git clone https://github.com/patryk27/kartoffel
      $ cd kartoffel
      $ ./build</pre
      >

      <p>
        ... and then go back, click <kbd>[upload bot]</kbd> and pick
        <kbd>./kartoffel</kbd>
      </p>

      <footer>
        <button @click="handleDisableSandboxHelp()">
          ok, don't show this again
        </button>
        <button @click="emit('close')">ok, got it</button>
      </footer>
    </template>

    <template v-else> TODO </template>
  </dialog>
</template>

<style scoped>
.game-help {
  max-width: 768px;

  p {
    &:last-child {
      margin-bottom: 0;
    }
  }

  ul {
    li {
      margin-bottom: 0.75em;
    }
  }

  footer {
    display: flex;
    justify-content: space-between;
  }
}
</style>
