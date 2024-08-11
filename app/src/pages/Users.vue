<script setup lang="ts">
import {PropType} from "vue";
import {SessionData} from "~/common/transport/connection.ts";

defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})
</script>

<template>
  <div>
    users
  </div>
</template>

<style scoped>

</style>
