<script setup lang="ts">
const emit = defineEmits<{
  close: [];
}>();

function handleDisable(): void {
  localStorage.setItem("sandbox-intro.popup", "disabled");
  emit("close");
}
</script>

<script lang="ts">
export function canOpen(): boolean {
  return localStorage.getItem("sandbox-intro.popup") == null;
}
</script>

<template>
  <dialog class="game-sandbox-intro">
    <nav>
      <div class="dialog-title">sandbox intro</div>
    </nav>

    <main>
      <p>hey there soldier and welcome to the sandbox 🫡🫡🫡</p>

      <p>
        sandbox is a special, private world where you can experiment with your
        bots before uploading them online
      </p>

      <p>as compared to the online play, in here:</p>

      <ul class="compact">
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

      <p>not sure what all this means? no problem, i don't either!</p>

      <p>
        anyway, click <kbd>[help]</kbd> button (top-right corner) to get more
        info
      </p>
    </main>

    <footer>
      <button @click="emit('close')">ok, got it</button>

      <button class="highlighted" @click="handleDisable()">
        ok, don't show this again
      </button>
    </footer>
  </dialog>
</template>

<style scoped>
.game-sandbox-intro {
  max-width: 768px;

  footer {
    display: flex;
    justify-content: space-between;
  }
}
</style>
