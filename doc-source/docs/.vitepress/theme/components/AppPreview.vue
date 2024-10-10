<script setup>
import {useData} from "vitepress"
import {onMounted, ref} from 'vue'

const {page, isDark, lang} = useData()

const loaded = ref(false)
onMounted(() => {
  loaded.value = true
})
</script>

<template>
  <div class="preview-wrapper">
    <div class="preview-inner">
      <video v-if="loaded"
             src="/video/etcd-workbench.mp4"
             autoplay
             muted
             loop
             class="preview-video"/>
    </div>
  </div>
</template>

<style scoped>
.preview-wrapper {
  margin: 0 0 50px 0;
  padding: 0;
  text-align: center;
  display: flex;
  flex-direction: row;
  justify-content: center;
}

.preview-inner {
  object-fit: cover;
  max-width: 100%;
  box-shadow: 0 0 10px 1px rgba(0,0,0,.6);
}

.preview-video, .preview-inner {
  border-radius: 3px;
}

@media (max-width: 639px) {
  .preview-inner {
    margin: 50px 30px;
  }
}

@media (min-width: 640px) {
  .preview-inner {
    margin: 50px 30px;
  }
}

@media (min-width: 960px) {
  .preview-inner {
    max-width: 1080px;
  }
}
</style>