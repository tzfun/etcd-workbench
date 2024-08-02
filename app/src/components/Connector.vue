<script setup lang="ts">
import {ref} from "vue";
import {ConnectionForm, ConnectionSshForm, ConnectionTlsForm, DefaultConnection} from "~/common/types.ts";
import etcdLogo from '~/assets/etcd.png'
import SingleFileSelector from "~/components/SingleFileSelector.vue";
import {Connection, SessionData, SshIdentity} from "~/common/transport/connection.ts";
import {_nonEmpty} from "~/common/utils.ts";
import {_connect, _connectTest} from "~/common/services.ts";
import {_loading, _tipError, _tipSuccess} from "~/common/events.ts";

const formData = ref<ConnectionForm>(JSON.parse(JSON.stringify(DefaultConnection)))
const formRules = ref({
  host: [
    (v?: string) => !!v || 'Host is required',
    (v: string) => {
      let regexIP = /^((25[0-5]|2[0-4]\d|((1\d{2})|([1-9]?\d)))\.){3}(25[0-5]|2[0-4]\d|((1\d{2})|([1-9]?\d)))$/;
      if (regexIP.test(v)) {
        return true
      } else if (v.toLowerCase() === 'localhost') {
        return true
      } else {
        return 'Invalid host'
      }
    }
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
      (v?: string) => {
        if (formData.value.tls.enable && v && v.length > 0) {
          let regexIP = /^((25[0-5]|2[0-4]\d|((1\d{2})|([1-9]?\d)))\.){3}(25[0-5]|2[0-4]\d|((1\d{2})|([1-9]?\d)))$/;
          if (regexIP.test(v)) {
            return true
          } else if (v.toLowerCase() === 'localhost') {
            return true
          } else {
            return 'Invalid authority'
          }
        }
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

const checkForm = async (): Connection => {
  const {valid} = await (formRef.value as HTMLFormElement).validate()
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

    let encoder = new TextEncoder()
    let tlsForm: ConnectionTlsForm = formData.value.tls
    if (tlsForm.enable) {
      connection.tls = {
        domain: tlsForm.domain,
        cert: [encoder.encode(tlsForm.cert.content)]
      }

      if (tlsForm.identity.enable) {
        connection.tls.identity = {
          cert: encoder.encode(tlsForm.identity.cert.content),
          key: encoder.encode(tlsForm.identity.key.content)
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
              key: encoder.encode(sshForm.identity.key.key.content)
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

const resetForm = () => {
  (formRef.value as HTMLFormElement).reset()
}

const testConnect = () => {
  checkForm().then((connection: Connection) => {
    console.log("test connection:", connection)
    _loading(true)
    _connectTest(connection).then(() => {
      _tipSuccess("Succeeded!")
    }).catch(e => {
      console.error(e)
      _tipError(`Failed: ${e}`)
    }).finally(() => {
      _loading(false)
    })
  }).catch(() => {

  })
}

const connect = () => {
  checkForm().then((connection: Connection) => {
    console.log("connection:", connection)
    _loading(true)
    _connect(connection).then((data: SessionData) => {
      console.log(data)
    }).catch(e => {
      console.error(e)
      _tipError(`Failed: ${e}`)
    }).finally(() => {
      _loading(false)
    })
  }).catch(() => {

  })
}
</script>

<template>
  <v-layout class="fill-height w-100 overflow-y-auto ml-0 mr-0 pl-0 pr-0 pt-12 pb-12">
    <div class="mx-auto my-auto">
      <div class="header">
        <div class="header-icon">
          <v-img :src="etcdLogo" cover/>
        </div>
        <h1 class="pt-0 pb-0 pl-5 header-title">ETCD Connection</h1>
      </div>
      <v-sheet class="justify-center mx-auto mt-5">
        <v-card width="600" class="connection-card">
          <v-card-text>
            <v-form ref="formRef">
              <div class="d-flex">
                <div class="form-label">
                  Connection Name
                </div>
                <div class="form-input">
                  <v-text-field
                      v-model="formData.name"
                      density="comfortable"
                      placeholder="New connection"
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
                      placeholder="127.0.0.1"
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
                        type="password"
                        density="comfortable"
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
                                        prompt-text="The file must be smaller than 128KB"
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
                                        prompt-text="The file must be smaller than 128KB"
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
                                        prompt-text="The file must be smaller than 128KB"
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
                        type="password"
                        density="comfortable"
                        placeholder="Password"
                    ></v-text-field>
                    <div v-else-if="formData.ssh.identity.model == 'key'">
                      <SingleFileSelector v-model="formData.ssh.identity.key.key"
                                          :max-size="128*1024"
                                          prompt-text="The file must be smaller than 128KB"
                      ></SingleFileSelector>

                      <v-text-field
                          v-model="formData.ssh.identity.key.passphrase"
                          type="password"
                          density="comfortable"
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
  box-shadow: 0 12px 32px 4px rgba(0, 0, 0, .04), 0 8px 20px rgba(0, 0, 0, .08);

  $--form-label-width: 150px;

  .form-label {
    width: $--form-label-width;
    line-height: 52px;
  }

  .form-label:after {
    content: ":";
  }

  .form-input {
    width: calc(100% - $--form-label-width);
  }
}
</style>