<script setup lang="ts">
import WindowsSystemBar from "~/components/system-bar/WindowsSystemBar.vue";

defineProps({
  height: Number,
  windowLabel: {
    type: String,
    required: true
  }
})
</script>

<template>
  <WindowsSystemBar :window-label="windowLabel" :height="height"></WindowsSystemBar>
</template>

<style scoped lang="scss">

</style>