<script setup lang="ts">
import {getCluster} from "~/services/SessionService";
import {MostlyCloudy, Refresh, User} from "@element-plus/icons-vue";

const props = defineProps({
  sessionKey: String
})

onMounted(() => {
  loadCluster()
})

const loadCluster = () => {
  getCluster(props.sessionKey).then(data => {
    console.log(data)
  })
}

const add = () => {

}

</script>

<template>
  <div>
    <div class="mb-5">
      <el-button @click="loadCluster" :icon="Refresh">Refresh Table</el-button>
      <el-button type="primary" :icon="MostlyCloudy" @click="add">Add Member</el-button>
    </div>
  </div>
</template>

<style lang="scss" scoped>

</style>