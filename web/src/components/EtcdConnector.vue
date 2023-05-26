<script lang="ts" setup>
import etcd from "~/assets/etcd.png"
import {ref} from "vue";
import type {UploadInstance, UploadProps, UploadRawFile} from "element-plus";
import {genFileId} from "element-plus";
import {Ref} from "@vue/reactivity";

const caFileInput = ref(null)

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
    caType: 'none',
    certMode: 'none',
    caFile: null,
    certFile: null,
    certKeyFile: null,
    password: ''
  }
})

const _testConnect = () => {
  console.log(form.value)
}

const _connect = () => {

}

const triggerClickCaInput = () => {

}

</script>
<template>
  <div class="container">
    <div class="header">
      <el-image style="width: 100px; height: 100px" :src="etcdLogo" fit="cover"/>
      <h1 class="header-title">Etcd Connection</h1>
    </div>
    <el-card class="box-card">
      <el-form v-model="form" :label-width="150" label-suffix=":">
        <el-form-item label="Session Name">
          <el-input v-model="form.name" placeholder="Please input session name"/>
        </el-form-item>
        <el-form-item label="Host">
          <el-input v-model="form.host" placeholder="Please input connect host"/>
        </el-form-item>
        <el-form-item label="Port">
          <el-input-number v-model="form.port" controls-position="right" placeholder="Please input connect port"/>
        </el-form-item>

        <el-divider content-position="center" class="divider">Authentication</el-divider>

        <el-form-item label="User">
          <el-input v-model="form.auth.username" placeholder="Please input authentication username"/>
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="form.auth.password" type="password" show-password
                    placeholder="Please input authentication password"/>
        </el-form-item>

        <el-divider content-position="center" class="divider">Certificate</el-divider>

        <el-form-item label="Cert Mode">
          <el-radio-group v-model="form.cert.caType">
            <el-radio label="none">None</el-radio>
            <el-radio label="public">Public CA</el-radio>
            <el-radio label="custom">Custom CA</el-radio>
          </el-radio-group>
        </el-form-item>

        <div v-show="form.cert.caType === 'custom'">
          <el-form-item label="CA File">
            <el-input type="file" class="hidden" ref="caFileInput"></el-input>
            <el-button type="primary" link @click="triggerClickCaInput">Select CA File</el-button>
          </el-form-item>

          <el-form-item label="Client Cert">
            <el-radio-group v-model="form.cert.certMode">
              <el-radio label="none">No Client Cert</el-radio>
              <el-radio label="password">Cert + Password</el-radio>
              <el-radio label="key">Cert + Key</el-radio>
            </el-radio-group>
          </el-form-item>

          <div v-show="form.cert.certMode !== 'none'">
            <el-form-item label="Cert File">
              <el-upload
                  ref="certFile"
                  :limit="1"
                  :on-exceed="handleCertFileExceed"
                  :auto-upload="false"
              >
                <template #trigger>
                  <el-button type="primary">select cert file</el-button>
                </template>
              </el-upload>
            </el-form-item>
          </div>

          <div v-show="form.cert.certMode === 'password'">
            <el-form-item label="Cert Password">
              <el-input type="password"
                        clearable
                        show-password
                        v-model="form.cert.password"
                        placeholder="Please input cert file password"></el-input>
            </el-form-item>
          </div>
          <div v-show="form.cert.certMode === 'key'">
            <el-form-item label="Cert Key File">
              <el-upload
                  ref="certKeyFile"
                  :limit="1"
                  :on-exceed="handleCertKeyFileExceed"
                  :auto-upload="false"
              >
                <template #trigger>
                  <el-button type="primary">select key file</el-button>
                </template>
              </el-upload>
            </el-form-item>
          </div>
        </div>

        <div style="margin: 35px 0">
          <el-button type="info" link @click="_testConnect">Test Connect</el-button>
          <el-button type="success" @click="_connect">Connect</el-button>
        </div>

      </el-form>
    </el-card>
  </div>

</template>

<style scoped>
.container {
  max-width: 500px;
  margin: 30px;
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