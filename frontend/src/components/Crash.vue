<script setup lang="ts">
defineProps<{
  msg: string;
}>();

function refresh() {
  location.reload();
}
</script>

<template>
  <main class="crash">
    <p>whoopsie, kartoffels have fell out of pot and ✨ crashed ✨:</p>

    <p>
      {{ msg }}
    </p>

    <p>
      please <a href="#" @click="refresh">refresh the page</a> and try again
    </p>
  </main>
</template>

<style scoped>
.crash {
  padding: 1em;
  border: 1px solid #ff0000;

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
