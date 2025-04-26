<script setup lang="ts">
import WindowsSystemBar from "~/components/system-bar/WindowsSystemBar.vue";
import {PropType} from "vue";
import {UpdateInfo} from "~/common/types.ts";

defineProps({
  height: Number,
  windowLabel: {
    type: String,
    required: true
  },
  updateInfo: {
    type: Object as PropType<UpdateInfo>,
    required: true
  }
})
</script>

<template>
  <WindowsSystemBar :window-label="windowLabel"
                    :height="height"
                    :update-info="updateInfo"
  />
</template>

<style scoped lang="scss">

</style>