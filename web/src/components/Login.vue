<script setup lang="ts">
import {reactive, ref} from "vue";
import {_isEmpty} from "~/util/Util";
import {login} from "~/service";
import {pushEvent} from "~/util/Event";
import {setToken, setUser} from "~/components/store";
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
  login(form.name, form.password).then((token: string) => {
    setUser(form.name)
    setToken(token)
    pushEvent('loginSuccess')
  }).catch((e: any) => {
    ElMessage({
      message: e,
      type: "warning",
      duration: 1500,
    });
  })
}
</script>

<template>
  <div class="login-container">
    <el-card class="login-box">
      <el-form reg="connectionForm" v-model="form" :label-width="100" label-suffix=":" label-position="right">
        <el-form-item label="Username">
          <el-input v-model="form.name" placeholder="Please input username"/>
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="form.password"
                    type="password"
                    show-password
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
.login-container {
  width: 100%;
  display: flex;
  justify-content: center;

  .login-box {
    max-width: 500px;
    margin-top: 10%;
  }
}
</style>