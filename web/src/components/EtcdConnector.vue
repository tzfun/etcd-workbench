<script lang="ts" setup>
import etcd from "~/assets/etcd.png"
import {Ref, ref} from "vue";
import {newSession, testSession} from "~/services/SessionService";
import {_isEmpty} from "~/util/BaseUtil";
import {ElMessage, UploadFile} from "element-plus";
import {NewSessionReq} from "~/entitys/RequestTypes";

const emits = defineEmits(["connected"])

const props = defineProps({
  checkSessionName: Function
})
const caFile = ref<UploadFile>()
const certFile = ref<UploadFile>()
const certKeyFile = ref<UploadFile>()

const etcdLogo = ref(etcd)
const form = ref({
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
  }
})

const _packFormData = async (): Promise<NewSessionReq> => {
  let msg: string

  const data: NewSessionReq = {
    caType: "",
    target: ""
  }

  data.namespace = _isEmpty(form.value.namespace) ? null : form.value.namespace

  if (!props.checkSessionName(form.value.name)) {
    msg = "Session name exists: " + form.value.name
  } else if (_isEmpty(form.value.host)) {
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
      if (!caFile.value) {
        msg = "Warning, please select CA file!"
      } else {
        let keyFileMaxSize = 24 * 1024;
        if (caFile.value?.size > keyFileMaxSize) {
          msg = "Warning, CA file is too large!"
        } else {
          data.caCert = await (caFile.value?.raw as File).text()
          if (certFile.value?.size > keyFileMaxSize) {
            msg = "Warning, Cert file is too large!"
          } else {
            if (!certFile.value) {
              msg = "Warning, please select client cert file!"
            } else {
              data.clientCert = await (certFile.value?.raw as File).text()
              if (form.value.cert.certMode === 'password') {
                data.clientCertPassword = form.value.cert.password
              } else if (form.value.cert.certMode === 'key') {
                if (!certKeyFile.value) {
                  msg = "Warning, please select client cert key file!"
                } else {
                  if (certKeyFile.value?.size > keyFileMaxSize) {
                    msg = "Warning, Cert key file is too large!"
                  } else {
                    data.clientCertKey = await (certKeyFile.value?.raw as File).text()
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  if (msg) {
    return Promise.reject(msg)
  } else {
    return Promise.resolve(data)
  }
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
      console.debug("Session connected ", res)
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
  console.debug("change", file)
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
  console.debug("remove", file)
}


</script>
<template>
  <div class="container">
    <div class="header">
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

.tip {
  color: #168f8f;
}
</style>