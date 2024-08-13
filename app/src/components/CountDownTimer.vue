<script setup lang="ts">
import {onMounted, onUnmounted, reactive, ref} from "vue";

const props = defineProps({
  value: {
    type: Number,
    required: true
  }
})

const timer = ref<number>()
const countdown = ref<number>(0)
const splitter = reactive({
  hour: 0,
  minute: 0,
  second: 0
})

onMounted(() => {
  countdown.value = props.value
  if (countdown.value > 0) {
    onTick()
    timer.value = setInterval(onTick, 1000)
  }
})

onUnmounted(() => {
  cleanTimer()
})

const onTick = () => {
  let v = --countdown.value;
  let hour = parseInt(v / 3600)
  v = v % 3600
  let minute = parseInt(v / 60)
  let second = parseInt(v % 60)

  splitter.hour = hour
  splitter.minute = minute
  splitter.second = second

  if (v <= 0) {
    cleanTimer()
  }
}

const cleanTimer = () => {
  let t = timer.value
  if (t) {
    timer.value = null
    clearInterval(t)
  }
}

</script>

<template>
  <span v-show="splitter.hour > 0">{{ splitter.hour }}h / </span>
  <span v-show="splitter.minute > 0">{{ splitter.minute }}m / </span>
  <span v-show="countdown > 0">{{ splitter.second }}s</span>
  <span v-if="countdown <= 0">Time Over</span>
</template>

<style scoped lang="scss">

</style>