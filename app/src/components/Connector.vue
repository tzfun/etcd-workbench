<script setup lang="ts">
import {PropType, reactive, ref, watch} from "vue";
import {ConnectionForm, ConnectionSshForm, ConnectionTlsForm, DefaultConnection} from "~/common/types.ts";
import SingleFileSelector from "~/components/SingleFileSelector.vue";
import {
  Connection,
  ConnectionInfo,
  ErrorPayload,
  KeyMonitor,
  SessionData,
  SshIdentity
} from "~/common/transport/connection.ts";
import {_decodeBytesToString, _encodeStringToBytes, _isEmpty, _nonEmpty} from "~/common/utils.ts";
import {_connect, _connectTest, _handleError, _saveConnection} from "~/common/services.ts";
import {_emitLocal, _loading, _tipSuccess, _tipWarn, EventName} from "~/common/events.ts";
import {VForm} from "vuetify/components";
import EtcdLogo from "~/components/EtcdLogo.vue";
import {trackEvent} from "~/common/analytics.ts";

const emits = defineEmits(['on-save'])
const props = defineProps({
  modelValue: {
    type: Object as PropType<ConnectionInfo>,
    required: true
  }
})

const formData = ref<ConnectionForm>(JSON.parse(JSON.stringify(DefaultConnection)))
const formRules = ref({
  host: [
    (v?: string) => !!v || 'Host is required'
  ],
  port: [
    (v?: string) => !!v || 'Port is required',
    (v: string) => {
      try {
        let num = parseInt(v)
        if (num <= 0 || num > 65535) {
          return 'Invalid port'
        }
        return true
      } catch (e) {
        return 'Invalid port'
      }
    }
  ],
  namespace: [
    (v?: string) => {
      if (v && v.length > 0) {
        if (v.startsWith("/")) {
          return true
        } else {
          return 'Namespace must be start with \'/\''
        }
      }
      return true
    }
  ],
  user: {
    username: [
      (v?: string) => {
        if (formData.value.user.enable) {
          return !!v || 'Username is required'
        }
        return true
      },
    ],
    password: [
      (v?: string) => {
        if (formData.value.user.enable) {
          return !!v || 'Password is required'
        }
        return true
      },
    ]
  },
  tls: {
    domain: [
      () => {
        return true
      }
    ]
  },
  ssh: {
    host: [
      (v?: string) => {
        if (formData.value.ssh.enable) {
          return !!v || 'SSH host is required'
        }
        return true
      },
      (v: string) => {
        if (formData.value.ssh.enable) {
          let regexIP = /^((25[0-5]|2[0-4]\d|((1\d{2})|([1-9]?\d)))\.){3}(25[0-5]|2[0-4]\d|((1\d{2})|([1-9]?\d)))$/;
          if (regexIP.test(v)) {
            return true
          } else if (v.toLowerCase() === 'localhost') {
            return true
          } else {
            return 'Invalid SSH host'
          }
        }
        return true
      }
    ],
    port: [
      (v?: string) => {
        if (formData.value.ssh.enable) {
          return !!v || 'SSH port is required'
        }
        return true
      },
      (v: string) => {
        if (formData.value.ssh.enable) {
          try {
            let num = parseInt(v)
            if (num <= 0 || num > 65535) {
              return 'Invalid SSH port'
            }
            return true
          } catch (e) {
            return 'Invalid SSH port'
          }
        }
        return true
      }
    ],
    user: [
      (v?: string) => {
        if (formData.value.ssh.enable) {
          return !!v || 'SSH user is required'
        }
        return true
      },
    ],
    identity: {
      password: [
        (v?: string) => {
          if (formData.value.ssh.identity.model == 'password') {
            return !!v || 'Password is required'
          }
          return true
        },
      ]
    }
  }
})
const formRef = ref(null)
const formPasswordShow = reactive({
  show1: false,
  show2: false,
  show3: false
})

watch(() => props.modelValue, (info: ConnectionInfo) => {
  let form: ConnectionForm = JSON.parse(JSON.stringify(DefaultConnection))
  if (!info.default) {
    form.name = info.name
    let connection: Connection = info.connection

    form.host = connection.host
    form.port = connection.port.toString()

    if (connection.namespace) {
      form.namespace = connection.namespace
    }

    let user = connection.user
    if (user) {
      form.user.enable = true
      form.user.username = user.username
      form.user.password = user.password
    }
    let tls = connection.tls
    if (tls) {
      form.tls.enable = true
      if (tls.cert.length > 0) {
        form.tls.cert.content = _decodeBytesToString(tls.cert[0])
      }
      if (tls.domain) {
        form.tls.domain = tls.domain
      }
      let identity = tls.identity
      if (identity) {
        form.tls.identity.enable = true
        form.tls.identity.cert.content = _decodeBytesToString(identity.cert)
        form.tls.identity.key.content = _decodeBytesToString(identity.key)
      }
    }

    let ssh = connection.ssh
    if (ssh) {
      form.ssh.enable = true
      form.ssh.host = ssh.host
      form.ssh.port = ssh.port.toString()
      form.ssh.user = ssh.user

      let identity = ssh.identity
      if (identity) {
        if (identity.password) {
          form.ssh.identity.model = 'password'
          form.ssh.identity.password = identity.password
        } else if (identity.key) {
          form.ssh.identity.model = 'key'
          form.ssh.identity.key.key.content = _decodeBytesToString(identity.key.key)

          let passphrase = identity.key.passphrase
          if (passphrase) {
            form.ssh.identity.key.passphrase = passphrase
          }
        }
      } else {
        form.ssh.identity.model = 'none'
      }
    }
  }
  formData.value = form
  resetFormValidation()
})

const checkForm = async (): Promise<Connection> => {
  const {valid} = await (formRef.value! as VForm).validate()
  if (valid) {
    let connection: Connection = {
      host: formData.value.host,
      port: parseInt(formData.value.port),
    }

    if (_nonEmpty(formData.value.namespace)) {
      connection.namespace = formData.value.namespace
    }

    if (formData.value.user.enable) {
      connection.user = {
        username: formData.value.user.username,
        password: formData.value.user.password
      }
    }

    let tlsForm: ConnectionTlsForm = formData.value.tls
    if (tlsForm.enable) {

      connection.tls = {
        domain: _isEmpty(tlsForm.domain) ? undefined : tlsForm.domain,
        cert: [_encodeStringToBytes(tlsForm.cert.content)]
      }

      if (tlsForm.identity.enable) {
        connection.tls.identity = {
          cert: _encodeStringToBytes(tlsForm.identity.cert.content),
          key: _encodeStringToBytes(tlsForm.identity.key.content)
        }
      }
    }

    let sshForm: ConnectionSshForm = formData.value.ssh
    if (sshForm.enable) {
      connection.ssh = {
        host: sshForm.host,
        port: parseInt(sshForm.port),
        user: sshForm.user,
      }
      switch (sshForm.identity.model) {
        case "password":
          connection.ssh.identity = {
            password: sshForm.identity.password
          }
          break
        case "key":
          let identity: SshIdentity = {
            key: {
              key: _encodeStringToBytes(sshForm.identity.key.key.content)
            }
          }

          if (_nonEmpty(sshForm.identity.key.passphrase)) {
            identity.key!.passphrase = sshForm.identity.key.passphrase
          }

          connection.ssh.identity = identity
          break
      }
    }

    return connection
  } else {
    throw new Error("Form invalid")
  }
}

const resetFormValidation = () => {
  if (formRef.value) {
    (formRef.value as VForm).resetValidation()
  }
}

const testConnect = () => {
  checkForm().then((connection: Connection) => {
    _loading(true, "Connection testing...")
    _connectTest(connection).then(() => {
      _tipSuccess("Succeeded!")
    }).catch((e: ErrorPayload | string) => {
      _handleError({
        e,
        prefix: "Failed: "
      })
    }).finally(() => {
      _loading(false)
    })
  }).catch(() => {

  })
}

const connect = () => {
  trackEvent('connect')
  checkForm().then((connection: Connection) => {
    let fd: ConnectionForm = formData.value;
    let name = fd.name
    if (_isEmpty(name)) {
      name = fd.host + ":" + fd.port
    }
    _loading(true, "Connecting")
    _connect(name, connection).then((session: SessionData) => {

      let keyCollection = session.keyCollection
      if (keyCollection) {
        session.keyCollectionSet = new Set<string>(keyCollection);
      } else {
        session.keyCollection = []
        session.keyCollectionSet = new Set<string>()
      }

      let keyMonitorList = session.keyMonitorList
      let keyMonitor: KeyMonitor = {
        list: [],
        map: {}
      }
      if (keyMonitorList) {
        for (let config of keyMonitorList) {
          keyMonitor.list.push(config.key)
          keyMonitor.map[config.key] = config
        }
      }
      session.keyMonitorList = undefined
      session.keyMonitor = keyMonitor

      _emitLocal(EventName.NEW_CONNECTION, {name, session})
    }).catch((e: ErrorPayload | string) => {
      _handleError({
        e,
        prefix: "Failed: "
      })
    }).finally(() => {
      _loading(false)
    })
  }).catch(() => {

  })
}

const saveConnection = () => {
  trackEvent('save_connection')
  let name = formData.value.name
  if (_isEmpty(name)) {
    _tipWarn("Connection name can not be empty")
    return
  }
  checkForm().then(connection => {
    _saveConnection(name, connection).then(() => {
      emits('on-save')
    }).catch((e: ErrorPayload | string) => {
      _handleError({e})
    })
  }).catch(() => {

  })
}
</script>

<template>
  <v-layout class="fill-height w-100 overflow-y-auto ml-0 mr-0 pl-0 pr-0 pt-12 pb-12">
    <div class="mx-auto my-auto">
      <div class="header user-select-none cursor-default">
        <div class="header-icon">
          <EtcdLogo :width="100" :height="100"></EtcdLogo>
        </div>
        <h1 class="pt-0 pb-0 pl-5 header-title">Server Connection</h1>
      </div>
      <v-sheet class="justify-center mx-auto mt-5">
        <v-card width="600" class="connection-card card-box-shadow" border>
          <v-card-text>
            <v-form ref="formRef" validate-on="submit lazy">
              <div class="d-flex">
                <div class="form-label">
                  Name
                </div>
                <div class="form-input">
                  <v-text-field
                      v-model="formData.name"
                      density="comfortable"
                      placeholder="Customize the name of the current connection"
                  ></v-text-field>
                </div>
              </div>

              <div class="d-flex">
                <div class="form-label">
                  Host
                </div>
                <div class="form-input">
                  <v-text-field
                      v-model="formData.host"
                      :rules="formRules.host"
                      density="comfortable"
                      placeholder="Etcd server host"
                  ></v-text-field>
                </div>
              </div>

              <div class="d-flex">
                <div class="form-label">
                  Port
                </div>
                <div class="form-input">
                  <v-text-field
                      v-model="formData.port"
                      :rules="formRules.port"
                      type="number"
                      density="comfortable"
                      placeholder="2379"
                  ></v-text-field>
                </div>
              </div>

              <div class="d-flex">
                <div class="form-label">
                  Namespace
                </div>
                <div class="form-input">
                  <v-text-field
                      v-model="formData.namespace"
                      :rules="formRules.namespace"
                      density="comfortable"
                      placeholder="Default is empty"
                  ></v-text-field>
                </div>
              </div>

              <v-row>
                <v-col class="align-content-center">
                  <v-checkbox label="Auth" v-model="formData.user.enable"></v-checkbox>
                </v-col>
                <v-col>
                  <v-checkbox label="SSL" v-model="formData.tls.enable"></v-checkbox>
                </v-col>
                <v-col>
                  <v-checkbox label="SSH" v-model="formData.ssh.enable"></v-checkbox>
                </v-col>
              </v-row>

              <v-sheet v-show="formData.user.enable">
                <v-divider>Authentication</v-divider>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    Username
                  </div>
                  <div class="form-input">
                    <v-text-field
                        v-model="formData.user.username"
                        :rules="formRules.user.username"
                        density="comfortable"
                        placeholder="Etcd auth username"
                    ></v-text-field>
                  </div>
                </div>

                <div class="d-flex">
                  <div class="form-label">
                    Password
                  </div>
                  <div class="form-input">
                    <v-text-field
                        v-model="formData.user.password"
                        :rules="formRules.user.password"
                        :type="formPasswordShow.show1 ? 'text' : 'password'"
                        :append-inner-icon="formPasswordShow.show1 ? 'mdi-eye-off' : 'mdi-eye'"
                        @click:append-inner="formPasswordShow.show1 = !formPasswordShow.show1"
                        density="comfortable"
                        autocomplete
                        placeholder="Etcd auth password"
                    ></v-text-field>
                  </div>
                </div>
              </v-sheet>

              <v-sheet v-show="formData.tls.enable">
                <v-divider>SSL/TLS</v-divider>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    Authority
                  </div>
                  <div class="form-input">
                    <v-text-field
                        v-model="formData.tls.domain"
                        :rules="formRules.tls.domain"
                        density="comfortable"
                        placeholder="Domain"
                    ></v-text-field>
                  </div>
                </div>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    CA File
                  </div>
                  <div class="form-input">
                    <SingleFileSelector v-model="formData.tls.cert"
                                        :max-size="128*1024"
                                        prompt-text="PEM encoded X509 certificate, less than 128KB."
                    ></SingleFileSelector>
                  </div>
                </div>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    Identity
                  </div>
                  <div class="form-input">
                    <v-checkbox v-model="formData.tls.identity.enable" label="Enable"></v-checkbox>
                  </div>
                </div>

                <div class="d-flex mt-5" v-if="formData.tls.identity.enable">
                  <div class="form-label">
                    Cert File
                  </div>
                  <div class="form-input">
                    <SingleFileSelector v-model="formData.tls.identity.cert"
                                        :max-size="128*1024"
                                        prompt-text="PEM encoded certificate, less than 128KB."
                    ></SingleFileSelector>
                  </div>
                </div>

                <div class="d-flex mt-5" v-if="formData.tls.identity.enable">
                  <div class="form-label">
                    Cert Key File
                  </div>
                  <div class="form-input">
                    <SingleFileSelector v-model="formData.tls.identity.key"
                                        :max-size="128*1024"
                                        prompt-text="PEM encoded private key, less than 128KB."
                    ></SingleFileSelector>
                  </div>
                </div>
              </v-sheet>

              <v-sheet v-show="formData.ssh.enable">
                <v-divider>SSH Tunnel</v-divider>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    Host
                  </div>
                  <div class="form-input">
                    <v-text-field
                        v-model="formData.ssh.host"
                        :rules="formRules.ssh.host"
                        density="comfortable"
                        placeholder="Host"
                    ></v-text-field>
                  </div>
                </div>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    Port
                  </div>
                  <div class="form-input">
                    <v-text-field
                        v-model="formData.ssh.port"
                        :rules="formRules.ssh.port"
                        type="number"
                        density="comfortable"
                        placeholder="Port"
                    ></v-text-field>
                  </div>
                </div>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    User
                  </div>
                  <div class="form-input">
                    <v-text-field
                        v-model="formData.ssh.user"
                        :rules="formRules.ssh.user"
                        density="comfortable"
                        placeholder="User"
                    ></v-text-field>
                  </div>
                </div>

                <div class="d-flex mt-5">
                  <div class="form-label">
                    Identity
                  </div>
                  <div class="form-input">
                    <v-radio-group v-model="formData.ssh.identity.model"
                                   inline
                    >
                      <v-radio
                          label="None"
                          value="none"
                      ></v-radio>
                      <v-radio
                          class="ml-2"
                          label="Password"
                          value="password"
                      ></v-radio>
                      <v-radio
                          class="ml-2"
                          label="Private Key"
                          value="key"
                      ></v-radio>
                    </v-radio-group>

                    <v-text-field
                        v-if="formData.ssh.identity.model == 'password'"
                        v-model="formData.ssh.identity.password"
                        :rules="formRules.ssh.identity.password"
                        :type="formPasswordShow.show2 ? 'text' : 'password'"
                        :append-inner-icon="formPasswordShow.show2 ? 'mdi-eye-off' : 'mdi-eye'"
                        @click:append-inner="formPasswordShow.show2 = !formPasswordShow.show2"
                        density="comfortable"
                        autocomplete
                        placeholder="Password"
                    ></v-text-field>
                    <div v-else-if="formData.ssh.identity.model == 'key'">
                      <SingleFileSelector v-model="formData.ssh.identity.key.key"
                                          :max-size="128*1024"
                                          prompt-text="Supported private key formats: Openssh, RSA, PKCS8, PKCS8 Encrypted. Less than 128KB."
                      ></SingleFileSelector>

                      <v-text-field
                          v-model="formData.ssh.identity.key.passphrase"
                          :type="formPasswordShow.show3 ? 'text' : 'password'"
                          :append-inner-icon="formPasswordShow.show3 ? 'mdi-eye-off' : 'mdi-eye'"
                          @click:append-inner="formPasswordShow.show3 = !formPasswordShow.show3"
                          density="comfortable"
                          autocomplete
                          placeholder="Passphrase (optional)"
                      ></v-text-field>
                    </div>
                  </div>
                </div>
              </v-sheet>
            </v-form>

            <div class="text-center pt-7 pb-7">
              <v-btn class="mt-2 pa-0 text-capitalize"
                     variant="text"
                     :ripple="false"
                     color="primary"
                     @click="testConnect"
              >Test Connect
              </v-btn>
              <v-btn class="mt-2 ml-4 text-capitalize"
                     variant="outlined"
                     @click="saveConnection"
              >Save to Favorites
              </v-btn>
              <v-btn class="mt-2 ml-4 text-capitalize"
                     color="blue-darken-1"
                     @click="connect"
              >Connect
              </v-btn>
            </div>

          </v-card-text>
        </v-card>
      </v-sheet>
    </div>
  </v-layout>
</template>

<style scoped lang="scss">
.header {
  display: flex;
  justify-content: center;

  $--icon-width: 100px;

  .header-icon {
    width: $--icon-width;
    height: $--icon-width;
  }

  .header-title {
    line-height: $--icon-width;
  }
}

.connection-card {
  $--form-label-width: 120px;

  .form-label {
    width: $--form-label-width;
    line-height: 52px;
    user-select: none;
  }

  .form-label:after {
    content: ":";
  }

  .form-input {
    width: calc(100% - $--form-label-width);
  }
}
</style>
