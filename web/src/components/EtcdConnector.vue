<script lang="ts" setup>
import etcd from "~/assets/etcd.png"
import {Ref, ref} from "vue";
import {newSession, testSession} from "~/services/SessionService";
import {_isEmpty} from "~/util/Util";
import {ElMessage, UploadFile} from "element-plus";
import {NewSessionReq} from "~/entitys/RequestTypes";
import {SessionStoreConfig} from "~/entitys/TransformTypes";

const emits = defineEmits(["connected","save"])

const props = defineProps({
  checkSessionName: Function
})
const caFile = ref<UploadFile>()
const certFile = ref<UploadFile>()
const certKeyFile = ref<UploadFile>()
const defaultForm = {
  name: 'localhost',
  host: '127.0.0.1',
  port: 2379,
  namespace: '',
  auth: {
    username: <string | null>null,
    password: <string | null>null
  },
  cert: {
    caType: 'none',
    certMode: 'none',
    password: <string | null>null,
    authority: <string | null>null
  },
  caCert: null,
  clientCert: null,
  clientCertKey: null
}

const etcdLogo = ref(etcd)
const form = ref(JSON.parse(JSON.stringify(defaultForm)))

const _packFormData = async (): Promise<NewSessionReq> => {
  const data: NewSessionReq = {
    caType: "",
    target: ""
  }

  data.namespace = _isEmpty(form.value.namespace) ? null : form.value.namespace

  if (!props.checkSessionName(form.value.name)) {
    return Promise.reject("Session name exists: " + form.value.name)
  } else if (_isEmpty(form.value.host)) {
    return Promise.reject('Warning, host can not be empty!')
  } else if (form.value.port <= 0) {
    return Promise.reject('Warning, invalid port!')
  } else {
    data.target = `ip:///${form.value.host}:${form.value.port}`
    data.user = form.value.auth.username
    data.password = form.value.auth.password
    data.authority = form.value.cert.authority
    data.caType = form.value.cert.caType

    if (form.value.cert.caType === 'custom') {

      const keyFileMaxSize = 24 * 1024;

      //  读取 CA File
      if (caFile.value || form.value.caCert) {
        //  从文件读caFile
        if (caFile.value) {
          if (caFile.value?.size >= keyFileMaxSize) {
            return Promise.reject("Warning, CA file is too large!")
          } else {
            data.caCert = await (caFile.value?.raw as File).text()
          }
        } else {
          data.caCert = form.value.caCert
        }
      } else {
        return Promise.reject("Warning, please select CA file!")
      }

      //  读取 Client Cert File
      if (certFile.value || form.value.clientCert) {
        if (certFile.value) {
          if (certFile.value?.size >= keyFileMaxSize) {
            return Promise.reject("Warning, Cert file is too large!")
          } else {
            data.clientCert = await (certFile.value?.raw as File).text()
          }
        } else {
          data.clientCert = form.value.clientCert
        }
      } else {
        return Promise.reject("Warning, please select client cert file!")
      }

      //  读取Client Cert Auth
      if (form.value.cert.certMode === 'password') {
        data.clientCertPassword = form.value.cert.password
      } else if (form.value.cert.certMode === 'key') {
        if (certKeyFile.value || form.value.clientCertKey) {
          if (certKeyFile.value) {
            if (certKeyFile.value?.size >= keyFileMaxSize) {
              return Promise.reject("Warning, Cert key file is too large!")
            } else {
              data.clientCertKey = await (certKeyFile.value?.raw as File).text()
            }
          } else {
            data.clientCertKey = form.value.clientCertKey
          }
        } else {
          return Promise.reject("Warning, please select client cert key file!")
        }
      }
    }
  }
  return Promise.resolve(data)
}

const _resetForm = () => {
  form.value = JSON.parse(JSON.stringify(defaultForm))
}

const _testConnect = () => {
  _packFormData().then(formData => {
    testSession(formData).then(res => {
      ElMessage({
        showClose: true,
        message: "Connect successful!",
        type: 'success',
      })
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
  _packFormData().then(formData => {
    newSession(formData).then(res => {
      emits('connected', {key: res, name: form.value.name})
    })
  }).catch(e => {
    ElMessage({
      showClose: true,
      message: e,
      type: 'warning',
      duration: 5000,
    })
  })
}

const _saveSessionConfig = () => {
  _packFormData().then((data: NewSessionReq) => {
    const sessionConfig: SessionStoreConfig = {
      name: form.value.name,
      host: form.value.host,
      port: form.value.port,
      namespace: data.namespace,
      user: data.user,
      password: data.password,
      authority: data.authority,
      caType: data.caType,
      caCert: data.caCert,
      clientCert: data.clientCert,
      clientCertMode: data.clientCertMode,
      clientCertPassword: data.clientCertPassword,
      clientCertKey: data.clientCertKey
    }

    emits('save', sessionConfig)
  }).catch(e => {
    ElMessage({
      showClose: true,
      message: e,
      type: 'warning',
    })
  })
}

const loadSessionConfig = (config: SessionStoreConfig) => {
  _resetForm()
  form.value.name = config.name
  form.value.host = config.host
  form.value.port = config.port
  form.value.namespace = config.namespace
  form.value.auth.username = config.user
  form.value.auth.password = config.password
  form.value.cert.caType = config.caType
  form.value.cert.authority = config.authority
  form.value.cert.certMode = config.clientCertMode
  form.value.cert.password = config.clientCertPassword
  form.value.caCert = config.caCert
  form.value.clientCert = config.clientCert
  form.value.clientCertKey = config.clientCertKey
}

const caFileChange = (file: UploadFile) => {
  fileChange(file, caFile)
}

const certFileChange = (file: UploadFile) => {
  fileChange(file, certFile)
}

const certKeyFileChange = (file: UploadFile) => {
  fileChange(file, certKeyFile)
}

const fileChange = (file: UploadFile, ref: Ref<UploadFile | undefined>) => {
  ref.value = file
}

const caFileRemove = (file: UploadFile) => {
  fileRemove(file, caFile)
}

const certFileRemove = (file: UploadFile) => {
  fileRemove(file, certFile)
}

const certKeyFileRemove = (file: UploadFile) => {
  fileRemove(file, certKeyFile)
}

const fileRemove = (file: UploadFile, ref: Ref<UploadFile | undefined>) => {
  ref.value = undefined
}

defineExpose({
  loadSessionConfig,
  resetSessionConfig: _resetForm
})

</script>
<template>
  <div class="container">
    <div class="header mb-4 mt-4">
      <el-image style="width: 100px; height: 100px" :src="etcdLogo" fit="cover"/>
      <h1 class="header-title">Etcd Connection</h1>
    </div>
    <el-card class="box-card">
      <el-form reg="connectionForm" v-model="form" :label-width="150" label-suffix=":">
        <el-form-item label="Session Name">
          <el-input v-model="form.name" placeholder="Please input session name"/>
        </el-form-item>
        <el-form-item label="Host">
          <el-input v-model="form.host" placeholder="Please input connect host"/>
        </el-form-item>
        <el-form-item label="Port">
          <el-input-number v-model="form.port" controls-position="right" placeholder="Please input connect port"/>
        </el-form-item>
        <el-form-item label="Namespace">
          <el-input v-model="form.namespace" placeholder="Default is empty"/>
        </el-form-item>

        <el-divider content-position="center" class="divider">Authentication</el-divider>

        <el-form-item label="User">
          <el-input v-model="form.auth.username" placeholder="Please input authentication username"/>
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="form.auth.password"
                    type="password"
                    show-password
                    autocomplete="off"
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
            <el-upload
                :limit="1"
                :auto-upload="false"
                :on-change="caFileChange"
                :on-remove="caFileRemove"
            >
              <template #trigger>
                <el-button type="primary" link>Select CA File</el-button>
              </template>
              <template #tip>
                <div class="el-upload__tip tip">
                  Key file with a size less than 24kb
                </div>
              </template>
            </el-upload>
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
              <el-upload
                  :limit="1"
                  :auto-upload="false"
                  :on-change="certFileChange"
                  :on-remove="certFileRemove"
              >
                <template #trigger>
                  <el-button type="primary" link>Select Cert File</el-button>
                </template>
                <template #tip>
                  <div class="el-upload__tip tip">
                    Key file with a size less than 24kb
                  </div>
                </template>
              </el-upload>
            </el-form-item>
          </div>

          <div v-show="form.cert.certMode === 'password'">
            <el-form-item label="Cert Password">
              <el-input type="password"
                        clearable
                        show-password
                        autocomplete="off"
                        v-model="form.cert.password"
                        placeholder="Please input cert file password"></el-input>
            </el-form-item>
          </div>
          <div v-show="form.cert.certMode === 'key'">
            <el-form-item label="Cert Key File">
              <el-upload
                  :limit="1"
                  :auto-upload="false"
                  :on-change="certKeyFileChange"
                  :on-remove="certKeyFileRemove"
              >
                <template #trigger>
                  <el-button type="primary" link>Select Cert Key File</el-button>
                </template>
                <template #tip>
                  <div class="el-upload__tip tip">
                    Key file with a size less than 24kb
                  </div>
                </template>
              </el-upload>
            </el-form-item>
          </div>
        </div>

        <div style="margin: 35px 0;text-align: center">
          <el-button type="primary" link @click="_testConnect">Test Connect</el-button>
          <el-button type="success" @click="_connect">Connect</el-button>
          <el-button plain @click="_saveSessionConfig">Save</el-button>
        </div>

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

.tip {
  color: #168f8f;
}
</style>