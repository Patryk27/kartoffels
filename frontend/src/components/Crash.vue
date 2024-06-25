<script setup>
  const props = defineProps(['msg']);

  function refresh() {
    location.reload();
  }
</script>

<template>
  <main class="component">
    <p>
      whoopsie, kartoffels have fell out of pot and ✨ crashed ✨:
    </p>

    <p>
      {{ msg }}
    </p>

    <p>
      please <a href="#" @click="refresh">refresh the page</a> and try again
    </p>
  </main>
</template>

<style scoped>
  .component {
    padding: 1em;
    border: 1px solid #ff0000;
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
