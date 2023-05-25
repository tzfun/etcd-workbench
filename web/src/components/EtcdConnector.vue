<script lang="ts" setup>
import etcd from "~/assets/etcd.png"
import {ref} from "vue";

const etcdLogo = ref(etcd)
const form = ref({
  name: 'localhost',
  host: '127.0.0.1',
  port: 2379,
  auth: {
    username: '',
    password: ''
  },
  cert: {

  }
})
</script>
<template>
  <div class="container">
    <div class="header">
      <el-image style="width: 100px; height: 100px" :src="etcdLogo" fit="cover" />
      <h1 class="header-title">Etcd Connection</h1>
    </div>
    <el-card class="box-card">
      <el-form v-model="form" :label-width="150">
        <el-form-item label="Session Name">
          <el-input v-model="form.name" placeholder="Please input session name" />
        </el-form-item>
        <el-form-item label="Host">
          <el-input v-model="form.host" placeholder="Please input connect host" />
        </el-form-item>
        <el-form-item label="Port">
          <el-input-number v-model="form.port" controls-position="right" placeholder="Please input connect port" />
        </el-form-item>

        <el-divider content-position="center" class="divider">Authentication</el-divider>

        <el-form-item label="User">
          <el-input v-model="form.auth.username" placeholder="Please input authentication username" />
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="form.auth.password" type="password" show-password placeholder="Please input authentication password" />
        </el-form-item>

        <el-divider content-position="center" class="divider">Cert</el-divider>

        <el-form-item label="User">
          <el-input v-model="form.auth.username" placeholder="Please input authentication username" />
        </el-form-item>
      </el-form>
    </el-card>
  </div>

</template>

<style scoped>
.container {
  max-width: 500px;
}
.header {
  display: flex;
  justify-content: center;
}

.header-title {
  padding: 0 26px;
}

.divider {
  margin: 40px 0;
}
</style>