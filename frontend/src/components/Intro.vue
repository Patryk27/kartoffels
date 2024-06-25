<script setup>
  const emit = defineEmits(['leave']);
</script>

<template>
  <main class="component">
    <p>
      hey there solder, you great piece of low-melting-point metal alloy
      🫡🫡🫡
    </p>

    <p>
      in kartoffels you're not just a spool of lead metal anymore, you're the
      lead programmer responsible for leading your bots into the victory
    </p>

    <p>
      here's a couple of things you need to know to get started:
    </p>

    <p>
      - maps are two-dimensional and vary in type (arena, dungeon etc.)
      <br>
      - robots are represented with the <i>@</i> sign (you'll see later)
      <br>
      - robots contain peripherals (radar, motors etc.) which your code can
        play with
      <br>
      - every tick each robot in randomized order executes a single CPU
        instruction
      <br>
      - there's 64k ticks per bot per second
      <br>
      - the game is simulated on a central server, so you're playing against
        other players
      <br>
      - also, the game is anonymous - let meritocracy thrive
    </p>

    <p>
      ---
    </p>

    <p>
      now, much too much talk, grug wants start
    </p>

    <pre>
      $ git clone github.com/patryk27/kartoffels-bot
      $ cd kartoffels-bot
      $ cargo build --release
    </pre>

    <p>
      having this ready, go back to the main page, select any map and start
      playing, improving your bot as you go!
    </p>

    <p>
      (( controls boil down to arrow keys (or WASD) and the up-front visible
      buttons ))
    </p>

    <div style="text-align: center; padding: 2.0em 0">
      <button @click="emit('leave')">
        i'm ready captain*
      </button>
    </div>

    <p style="color: #505050; font-style: italic; text-align: right">
      * kartoffels ltd is not responsible and can't be held liable for loss of
      hearing, loss of sight, spilling all your matcha onto the floor, sudden
      feeling of the flight and fight syndrome, wanting to do origami but
      being unable to etc. etc. etc.
    </p>
  </main>
</template>

<style scoped>
  .component {
    margin-top: 1em;
    padding: 1em;
    max-width: 1024px;
    border: 1px solid #ff8000;
    border-radius: 5px;

    p {
      &:first-child {
        margin-top: 0;
      }

      &:last-child {
        margin-bottom: 0;
      }
    }
  }
</style>
