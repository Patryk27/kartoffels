<script setup>
  import { ref, watch, onMounted } from 'vue';

  const emit = defineEmits(['worldChange', 'pause']);
  const props = defineProps(['session', 'paused']);
  const world = ref(props.session ? props.session.worldId : null);
  const worlds = ref([]);

  watch(world, value => {
    emit('worldChange', value);
  });

  onMounted(async () => {
    try {
      var response = await fetch(`${import.meta.env.VITE_HTTP_URL}/worlds`);
      var response = await response.json();

      worlds.value = response.worlds;

      if (world.value == null) {
        world.value = response.worlds[0].id;
      }
    } catch (err) {
      window.onerror(err);
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
          resume
        </template>

        <template v-else>
          pause
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
