<script lang="ts" setup>

import {ref} from "vue";

defineProps({
  key: String
})
const state = ref('new')
</script>

<template>
  <div v-if="state === 'new'" class="connector">
    <EtcdConnector></EtcdConnector>
  </div>
</template>

<style scoped>
.connector {
  display: flex;
  justify-content: center;
}
</style>