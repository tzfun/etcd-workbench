<script setup lang="ts">

defineProps({
  showResizeLine: {
    type: Boolean,
    default: true
  }
})
</script>

<template>
  <div class="d-flex drag-item">
    <div class="drag-item-content overflow-y-auto">
      <slot></slot>
    </div>
    <div v-if="showResizeLine" class="drag-resize-line"></div>
  </div>
</template>

<style scoped lang="scss">
.drag-item {
  min-width: 200px;
  position: relative;
}
.drag-item-content {
  width: 100%;
  height: 100%;
}
.drag-resize-line {
  position: absolute;
  top: 0;
  right: 0;
  width: 1px;
  height: 100%;
  cursor: col-resize;
}

.drag-resize-line:hover {
  width: 4px;
}

.v-theme--light {
  .drag-resize-line {
    background-color: #d6d6d6;
  }
  .drag-resize-line:hover {
    background-color: #9e9c9c;
  }
}

.v-theme--dark {
  .drag-resize-line {
    background-color: #4c4b4b;
  }

  .drag-resize-line:hover {
    background-color: #9e9c9c;
  }
}
</style>