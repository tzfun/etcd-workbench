<script setup lang="ts">
import {onMounted, onUnmounted, reactive, ref} from "vue";

const props = defineProps({
  value: {
    type: Number,
    required: true
  }
})

const timer = ref()
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
  let hour = Math.floor(v / 3600)
  v = v % 3600
  let minute = Math.floor(v / 60)
  let second = Math.floor(v % 60)

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
    timer.value = undefined
    clearInterval(t)
  }
}

</script>

<template>
  <v-icon class="mr-1" color="light-blue-accent-3">mdi-clock-time-four-outline</v-icon>
  <span v-show="splitter.hour > 0">{{ splitter.hour }}h / </span>
  <span v-show="splitter.minute > 0">{{ splitter.minute }}m / </span>
  <span v-show="countdown > 0">{{ splitter.second }}s</span>
  <span v-if="countdown <= 0">Time Over</span>
</template>

<style scoped lang="scss">

</style>