<script setup>
  import { ref, watch, onMounted } from 'vue';

  const emit = defineEmits(['worldChange', 'pause']);
  const props = defineProps(['world', 'paused']);
  const world = ref(null);
  const worlds = ref([]);

  watch(world, value => {
    emit('worldChange', value);
  });

  onMounted(async () => {
    var response = await fetch(`${import.meta.env.VITE_HTTP_URL}/worlds`);
    var response = await response.json();

    worlds.value = response.worlds;

    if (props.world) {
      world.value = props.world.id;
    } else {
      world.value = response.worlds[0].id;
    }
  });
</script>

<template>
  <nav>
    <div>
      <label for="world">
        world:
      </label>

      <select id="world" v-model="world">
        <option v-for="world in worlds" :value="world.id">
          {{ world.name }} ({{ world.mode }}; {{ world.theme }})
        </option>
      </select>
    </div>

    <div>
      <button
          id="pause"
          :class="paused ? 'paused' : ''"
          @click="emit('pause')">
        <template v-if="paused">
          ⏵︎
        </template>

        <template v-else>
          ⏸︎
        </template>
      </button>
    </div>
  </nav>
</template>

<style scoped>
  nav {
    display: flex;
    margin-bottom: 0.5em;

    >div:first-child {
      flex-grow: 1;
    }
  }

  #pause {
    &.paused {
      border: 1px solid red;
    }
  }
</style>
