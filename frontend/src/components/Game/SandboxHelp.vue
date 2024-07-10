<script setup lang="ts">
const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <dialog class="game-sandbox-help">
    <nav>
      <div class="dialog-title">sandbox help</div>

      <div class="dialog-buttons">
        <button @click="emit('close')">close</button>
      </div>
    </nav>

    <p>hey there solder, welcome to the sandbox 🫡🫡🫡</p>

    <p>
      sandbox is a special, private world where you can experiment with your
      bots before uploading them online
    </p>

    <p>as compared to the online play, in here:</p>

    <ul>
      <li>
        you can use the <kbd>[configure]</kbd> button (located above) to change
        how the world looks like
      </li>

      <li>
        you can spawn <kbd>roberto 🔪</kbd>, the kartoffels' built-in moderately
        challenging bot
      </li>

      <li>you can destroy and kill bots</li>
    </ul>

    <p>
      not sure what all this means? no problem, i don't either! -- the eggheads
      left some extra instructions, though:
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
  </dialog>
</template>

<style scoped>
.game-sandbox-help {
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

  pre {
    margin-top: 1em;
    padding-top: 1em;
    border-top: 1px dashed #00ff80;
  }
}
</style>
