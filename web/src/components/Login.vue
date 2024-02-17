<script setup lang="ts">
import {reactive} from "vue";
import {_isEmpty} from "~/common/Util";
import {_login} from "~/common/Service";
import {pushEvent} from "~/common/Event";
import {setToken, setUser} from "~/common/Store";
import {ElMessage} from "element-plus";

const form = reactive({
  name: '',
  password: ''
})

const tryLogin = () => {
  if (_isEmpty(form.name)) {
    return
  }
  if (_isEmpty(form.password)) {
    return
  }
  _login(form.name, form.password).then((token: string) => {
    setUser(form.name)
    setToken(token)
    pushEvent('loginSuccess')
  }).catch((e: any) => {
  })
}
</script>

<template>
  <div class="login-container">
    <h1 class="login-title">Login ETCD Workbench</h1>
    <el-card class="login-box">
      <el-form reg="connectionForm" v-model="form" :label-width="100" label-suffix=":" label-position="right">
        <el-form-item label="Username">
          <el-input v-model="form.name" placeholder="Please input username" @keyup.enter="tryLogin"/>
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="form.password"
                    type="password"
                    show-password
                    @keyup.enter="tryLogin"
                    placeholder="Please input password"/>
        </el-form-item>
        <div>
          <el-button type="primary" class="mt-2 mb-2 mt-4 mb-4" style="width: 100%" @click="tryLogin">Login</el-button>
        </div>
      </el-form>
    </el-card>
  </div>

</template>

<style scoped lang="scss">

$--login-container-width: 500px;

.login-container {
  width: $--login-container-width;
  position: relative;
  left: 50%;
  margin-left: calc($--login-container-width / -2);
  margin-top: 120px;

  .login-title {
    text-align: center;
  }

  .login-box {
    padding: 30px 0 15px 0;
  }
}

</style>
