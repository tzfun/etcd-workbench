<script setup lang="ts">
import {PropType} from "vue";
import {SessionData} from "~/common/transport/connection.ts";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})
</script>

<template>
  <div>
    roles
  </div>
</template>

<style scoped>

</style>