<script lang="ts" setup>
import etcd from "~/assets/etcd.png"
import {ref} from "vue";
import {testSession} from "~/services/SessionService";
import {_isEmpty} from "~/util/BaseUtil";
import {ElMessage} from "element-plus";
import {_loading} from "~/util/CommonUtil";

const caFileInput = ref(null)

const etcdLogo = ref(etcd)
const form = ref({
  name: 'localhost',
  host: '127.0.0.1',
  port: 2379,
  auth: {
    username: <string | null>null,
    password: <string | null>null
  },
  cert: {
    caType: 'none',
    certMode: 'none',
    caFile: <File | null>null,
    certFile: <File | null>null,
    certKeyFile: <File | null>null,
    password: <string | null>null,
    authority: <string | null>null
  }
})

const _packFormData = async (): Promise<object | string> => {
  const data = {
    target: '',
    user: <string | null>null,
    password: <string | null>null,
    authority: <string | null>null,
    caType: 'none',
    caCert: <string | null>null,
    clientCertMode: 'none',
    clientCert: <string | null>null,
    clientCertPassword: <string | null>null,
    clientCertKey: <string | null>null
  }
  let msg: string
  if (_isEmpty(form.value.host)) {
    msg = 'Warning, host can not be empty!'
  } else if (form.value.port <= 0) {
    msg = 'Warning, invalid port!'
  } else {
    data.target = `ip:///${form.value.host}:${form.value.port}`
    data.user = form.value.auth.username
    data.password = form.value.auth.password
    data.authority = form.value.cert.authority
    data.caType = form.value.cert.caType

    if (form.value.cert.caType === 'custom') {
      data.caCert = await form.value.cert.caFile?.text()
      if (form.value.cert.certMode === 'password') {
        data.clientCert = await form.value.cert.certFile?.text()
        data.clientCertPassword = form.value.cert.password
      } else if (form.value.cert.certMode === 'key') {
        data.clientCert = await form.value.cert.certFile?.text()
        data.clientCertKey = await form.value.cert.certKeyFile?.text()
      }
    }
    return Promise.resolve(data)
  }
  return Promise.reject(msg)
}

const _testConnect = () => {
  _packFormData().then(formData => {
    console.log(formData)
    let loading = _loading()
    testSession(formData).then(res => {
      ElMessage({
        showClose: true,
        message: "Connect successful!",
        type: 'success',
      })
    }).finally(() => {
      loading.close()
    })
  }).catch(e => {
    ElMessage({
      showClose: true,
      message: e,
      type: 'warning',
    })
  })
}

const _connect = () => {

}

const triggerClickInput = (type: string) => {

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

        <el-divider content-position="center" class="divider">SSL/TLS</el-divider>

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
            <el-button type="info" link @click="triggerClickInput('ca')">Select CA File</el-button>
          </el-form-item>

          <el-form-item label="Authority">
            <el-input v-model="form.cert.authority" placeholder="127.0.0.1"></el-input>
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
              <el-input type="file" class="hidden" ref="certFileInput"></el-input>
              <el-button type="info" link @click="triggerClickInput('cert')">Select Cert File</el-button>
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
              <el-input type="file" class="hidden" ref="certKeyFileInput"></el-input>
              <el-button type="info" link @click="triggerClickInput('certKey')">Select Cert Key File</el-button>
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