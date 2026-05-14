<script setup lang="ts">
import {PropType, reactive, ref, watch} from "vue";
import {ConnectionForm, ConnectionSshForm, ConnectionTlsForm, DefaultConnection} from "~/common/types.ts";
import SingleFileSelector from "~/components/SingleFileSelector.vue";
import {
  Connection,
  ConnectionInfo,
  ErrorPayload,
  HashAlgorithm,
  KeyMonitorConfig,
  SessionData,
  SshIdentity
} from "~/common/transport/connection.ts";
import {_decodeBytesToString, _encodeStringToBytes, _isEmpty, _nonEmpty} from "~/common/utils.ts";
import {_connect, _connectTest, _handleError, _saveConnection} from "~/common/services.ts";
import {_emitLocal, _loading, _tipSuccess, _tipWarn, EventName} from "~/common/events.ts";
import {VForm} from "vuetify/components";
import EtcdLogo from "~/components/EtcdLogo.vue";
import {trackEvent} from "~/common/analytics.ts";
import {useI18n} from "vue-i18n";

const {t} = useI18n()

const emits = defineEmits(['on-save'])
const props = defineProps({
  modelValue: {
    type: Object as PropType<ConnectionInfo>,
    required: true
  }
})
const logoRef = ref<InstanceType<typeof HTMLDivElement>>()

const formData = ref<ConnectionForm>(JSON.parse(JSON.stringify(DefaultConnection)))
const formRules = ref({
  host: [
    (v?: string) => !!v || t('main.home.connector.form.ruleHost')
  ],
  port: [
    (v?: string) => !!v || t('main.home.connector.form.rulePort'),
    (v: string) => {
      try {
        let num = parseInt(v)
        if (num <= 0 || num > 65535) {
          return t('main.home.connector.form.rulePortInvalid')
        }
        return true
      } catch (e) {
        return t('main.home.connector.form.rulePortInvalid')
      }
    }
  ],
  user: {
    username: [
      (v?: string) => {
        if (formData.value.user.enable) {
          return !!v || t('main.home.connector.form.ruleUsername')
        }
        return true
      },
    ],
    password: [
      (v?: string) => {
        if (formData.value.user.enable) {
          return !!v || t('main.home.connector.form.rulePassword')
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
          return !!v || t('main.home.connector.form.ruleSshHost')
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
            return t('main.home.connector.form.ruleHostInvalid')
          }
        }
        return true
      }
    ],
    port: [
      (v?: string) => {
        if (formData.value.ssh.enable) {
          return !!v || t('main.home.connector.form.rulePort')
        }
        return true
      },
      (v: string) => {
        if (formData.value.ssh.enable) {
          try {
            let num = parseInt(v)
            if (num <= 0 || num > 65535) {
              return t('main.home.connector.form.rulePortInvalid')
            }
            return true
          } catch (e) {
            return t('main.home.connector.form.rulePortInvalid')
          }
        }
        return true
      }
    ],
    user: [
      (v?: string) => {
        if (formData.value.ssh.enable) {
          return !!v || t('main.home.connector.form.ruleUser')
        }
        return true
      },
    ],
    identity: {
      password: [
        (v?: string) => {
          if (formData.value.ssh.identity.model == 'password') {
            return !!v || t('main.home.connector.form.rulePassword')
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

const tab = ref('general')

watch(() => props.modelValue, (info: ConnectionInfo) => {
  let form: ConnectionForm = JSON.parse(JSON.stringify(DefaultConnection))
  if (!info.default) {
    form.name = info.name
    let connection: Connection = info.connection

    form.host = connection.host
    form.port = connection.port.toString()
    form.queryPagination = connection.queryPagination == undefined ? true : connection.queryPagination
    form.queryPaginationSize = connection.queryPaginationSize ? connection.queryPaginationSize.toString() : '2000'

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
          let hashAlgorithm = identity.key.hashAlgorithm
          if (_isEmpty(hashAlgorithm)) {
            form.ssh.identity.key.hashAlgorithm = ""
          } else {
            form.ssh.identity.key.hashAlgorithm = hashAlgorithm
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
      queryPagination: formData.value.queryPagination,
      queryPaginationSize: parseInt(formData.value.queryPaginationSize),
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
              key: _encodeStringToBytes(sshForm.identity.key.key.content),
              hashAlgorithm: _isEmpty(sshForm.identity.key.hashAlgorithm) ? undefined : sshForm.identity.key.hashAlgorithm as HashAlgorithm
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
    _loading(true, t('main.home.connector.connectionTesting'))
    _connectTest(connection).then(() => {
      _tipSuccess(t('main.home.connector.testSuccess'))
    }).catch((e: ErrorPayload | string) => {
      _handleError({
        e,
        prefix: `${t('common.failed')}: `
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
    _loading(true, t('main.home.connector.connecting'))
    _connect(name, connection).then((session: SessionData) => {

      let keyCollection = session.keyCollection
      if (keyCollection) {
        session.keyCollectionSet = new Set<string>(keyCollection);
      } else {
        session.keyCollection = []
        session.keyCollectionSet = new Set<string>()
      }

      let keyMonitorList = session.keyMonitorList
      let keyMonitorMap: Record<string, KeyMonitorConfig> = {}
      if (keyMonitorList) {
        for (let config of keyMonitorList) {
          keyMonitorMap[config.key] = config
        }
      }
      session.keyMonitorList = undefined
      session.keyMonitorMap = keyMonitorMap

      _emitLocal(EventName.NEW_CONNECTION, {name, session})
    }).catch((e: ErrorPayload | string) => {
      _handleError({
        e,
        prefix: `${t('common.failed')}: `
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
    _tipWarn(t('main.home.connector.nameEmptyTip'))
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
const scrollToTop = () => {
  if (logoRef.value) {
    logoRef.value.scrollIntoView({
      behavior: "smooth",
      block: "center",
    })
  }
}

defineExpose({
  scrollToTop
})
</script>

<template>
  <v-layout class="fill-height w-100 overflow-y-auto ml-0 mr-0 pl-0 pr-0 pt-12 pb-12">
    <div class="mx-auto my-auto w-100 px-4" style="max-width: 900px;">
      <div class="header user-select-none cursor-default mb-6">
        <div class="header-icon" ref="logoRef">
          <EtcdLogo :width="100" :height="100"></EtcdLogo>
        </div>
        <h1 class="pt-0 pb-0 pl-5 header-title">{{ t("main.home.connector.serverConnection") }}</h1>
      </div>
      <v-sheet class="justify-center mx-auto mt-5 rounded-lg bg-transparent">
        <v-card class="connection-card card-box-shadow rounded-lg" border width="100%">

          <v-tabs v-model="tab" color="primary" align-tabs="start" class="border-b" height="60">
            <v-tab value="general" class="text-none">
              <v-icon start>mdi-tune</v-icon>
              {{ t('common.basic') }}
            </v-tab>
            <v-tab value="auth" class="text-none">
              <v-icon start :color="formData.user.enable ? 'success' : undefined">mdi-shield-account</v-icon>
              {{ t("main.home.connector.form.authDivider") }}
            </v-tab>
            <v-tab value="ssl" class="text-none">
              <v-icon start :color="formData.tls.enable ? 'success' : undefined">mdi-lock</v-icon>
              {{ t("main.home.connector.form.sslDivider") }}
            </v-tab>
            <v-tab value="ssh" class="text-none">
              <v-icon start :color="formData.ssh.enable ? 'success' : undefined">mdi-console-network</v-icon>
              {{ t('main.home.connector.form.sshDivider') }}
            </v-tab>
          </v-tabs>

          <v-card-text class="pa-6">
            <v-form ref="formRef" validate-on="submit lazy">

              <div class="tabs-window-container">

                <!-- General Tab -->
                <div v-show="tab === 'general'">
                  <div class="d-flex">
                    <div class="form-label">
                      {{ t("main.home.connector.form.name") }}
                    </div>
                    <div class="form-input">
                      <v-text-field
                          v-model="formData.name"
                          density="comfortable"
                          :placeholder="t('main.home.connector.form.namePlaceholder')"
                      ></v-text-field>
                    </div>
                  </div>

                  <div class="d-flex">
                    <div class="form-label">
                      {{ t("main.home.connector.form.host") }}
                    </div>
                    <div class="form-input">
                      <v-text-field
                          v-model="formData.host"
                          :rules="formRules.host"
                          density="comfortable"
                          :placeholder="t('main.home.connector.form.hostPlaceholder')"
                      ></v-text-field>
                    </div>
                  </div>

                  <div class="d-flex">
                    <div class="form-label">
                      {{ t("main.home.connector.form.port") }}
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
                      {{ t("main.home.connector.form.namespace") }}
                    </div>
                    <div class="form-input">
                      <v-text-field
                          v-model="formData.namespace"
                          density="comfortable"
                          :placeholder="t('main.home.connector.form.namespacePlaceholder')"
                      ></v-text-field>
                    </div>
                  </div>

                  <div class="d-flex mt-2 mb-6">
                    <div class="form-label form-radio-label d-flex align-center">
                      {{ t("main.home.connector.form.queryPagination") }}
                      <v-tooltip interactive location="top">
                        <template v-slot:activator="{ props: activatorProps }">
                          <v-icon
                              icon="mdi-information-outline"
                              v-bind="activatorProps"
                              size="small"
                              color="blue-lighten-1"
                              class="mx-1"
                          />
                        </template>
                        <div>
                          {{ t("main.home.connector.form.queryPaginationDetail") }}
                        </div>
                      </v-tooltip>
                    </div>
                    <div class="form-input">
                      <v-radio-group v-model="formData.queryPagination" inline hide-details>
                        <v-radio
                            class="ml-2"
                            :label="t('common.enable')"
                            :value="true"
                        ></v-radio>
                        <v-radio
                            :label="t('common.disable')"
                            :value="false"
                        ></v-radio>
                      </v-radio-group>
                    </div>
                  </div>

                  <div class="d-flex mt-4" v-if="formData.queryPagination">
                    <div class="form-label">
                      {{ t("main.home.connector.form.queryPaginationSize") }}
                    </div>
                    <div class="form-input">
                      <v-text-field
                          v-model="formData.queryPaginationSize"
                          type="number"
                          density="comfortable"
                          hide-details
                          :placeholder='t("main.home.connector.form.queryPaginationSizeDetail")'
                      />
                    </div>
                  </div>
                </div>

                <!-- Auth Tab -->
                <div v-show="tab === 'auth'">
                  <v-list-item
                      class="mb-6 rounded-lg border"
                      :class="formData.user.enable ? 'border-primary' : ''"
                      lines="one"
                  >
                    <template v-slot:prepend>
                      <v-avatar :color="formData.user.enable ? 'primary' : 'grey'" variant="tonal" rounded class="mr-3">
                        <v-icon>mdi-shield-account</v-icon>
                      </v-avatar>
                    </template>
                    <v-list-item-title class="font-weight-bold" :class="formData.user.enable ? 'text-primary' : ''">
                      {{ t('main.home.connector.form.authDivider') }}
                    </v-list-item-title>
                    <template v-slot:append>
                      <v-switch
                          v-model="formData.user.enable"
                          color="primary"
                          hide-details
                          inset
                      ></v-switch>
                    </template>
                  </v-list-item>

                  <div v-show="formData.user.enable">
                    <div class="d-flex mt-2">
                      <div class="form-label">
                        {{ t("main.home.connector.form.authUsername") }}
                      </div>
                      <div class="form-input">
                        <v-text-field
                            v-model="formData.user.username"
                            :rules="formRules.user.username"
                            density="comfortable"
                            :placeholder="t('main.home.connector.form.authUsernamePlaceholder')"
                        ></v-text-field>
                      </div>
                    </div>

                    <div class="d-flex">
                      <div class="form-label">
                        {{ t("common.password") }}
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
                            :placeholder="t('main.home.connector.form.authPasswordPlaceholder')"
                        ></v-text-field>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- SSL Tab -->
                <div v-show="tab === 'ssl'">
                  <v-list-item
                      class="mb-6 rounded-lg border"
                      :class="formData.tls.enable ? 'border-primary' : ''"
                      lines="one"
                  >
                    <template v-slot:prepend>
                      <v-avatar :color="formData.tls.enable ? 'primary' : 'grey'" variant="tonal" rounded class="mr-3">
                        <v-icon>mdi-lock</v-icon>
                      </v-avatar>
                    </template>
                    <v-list-item-title class="font-weight-bold" :class="formData.tls.enable ? 'text-primary' : ''">
                      {{ t('main.home.connector.form.sslDivider') }}
                    </v-list-item-title>
                    <template v-slot:append>
                      <v-switch
                          v-model="formData.tls.enable"
                          color="primary"
                          hide-details
                          inset
                      ></v-switch>
                    </template>
                  </v-list-item>

                  <div v-show="formData.tls.enable">
                    <div class="d-flex mt-2">
                      <div class="form-label">
                        {{ t("main.home.connector.form.sslAuthority") }}
                      </div>
                      <div class="form-input">
                        <v-text-field
                            v-model="formData.tls.domain"
                            :rules="formRules.tls.domain"
                            density="comfortable"
                            :placeholder="t('main.home.connector.form.sslAuthorityPlaceholder')"
                        ></v-text-field>
                      </div>
                    </div>

                    <div class="d-flex">
                      <div class="form-label">
                        {{ t("main.home.connector.form.sslCAFile") }}
                      </div>
                      <div class="form-input">
                        <SingleFileSelector
                            v-model="formData.tls.cert"
                            :max-size="128*1024"
                            :prompt-text="t('main.home.connector.form.sslCAFilePlaceholder')"
                        ></SingleFileSelector>
                      </div>
                    </div>

                    <div class="d-flex mt-4">
                      <div class="form-label form-checkbox-label">
                        {{ t('main.home.connector.form.identity') }}
                      </div>
                      <div class="form-input">
                        <v-checkbox
                            v-model="formData.tls.identity.enable"
                            :label="t('common.enable')"
                            hide-details
                        />
                      </div>
                    </div>

                    <v-expand-transition>
                      <div v-show="formData.tls.identity.enable" class="mt-4">
                        <div class="d-flex">
                          <div class="form-label">
                            {{ t('main.home.connector.form.certFile') }}
                          </div>
                          <div class="form-input">
                            <SingleFileSelector
                                v-model="formData.tls.identity.cert"
                                :max-size="128*1024"
                                :prompt-text="t('main.home.connector.form.certFilePlaceholder')"
                            ></SingleFileSelector>
                          </div>
                        </div>

                        <div class="d-flex">
                          <div class="form-label">
                            {{ t('main.home.connector.form.certKetFile') }}
                          </div>
                          <div class="form-input">
                            <SingleFileSelector
                                v-model="formData.tls.identity.key"
                                :max-size="128*1024"
                                :prompt-text="t('main.home.connector.form.certKetFilePlaceholder')"
                            ></SingleFileSelector>
                          </div>
                        </div>
                      </div>
                    </v-expand-transition>
                  </div>
                </div>

                <!-- SSH Tab -->
                <div v-show="tab === 'ssh'">
                  <v-list-item
                      class="mb-6 rounded-lg border"
                      :class="formData.ssh.enable ? 'border-primary' : ''"
                      lines="one"
                  >
                    <template v-slot:prepend>
                      <v-avatar :color="formData.ssh.enable ? 'primary' : 'grey'" variant="tonal" rounded class="mr-3">
                        <v-icon>mdi-console-network</v-icon>
                      </v-avatar>
                    </template>
                    <v-list-item-title class="font-weight-bold" :class="formData.ssh.enable ? 'text-primary' : ''">
                      {{ t('main.home.connector.form.sshDivider') }}
                    </v-list-item-title>
                    <template v-slot:append>
                      <v-switch
                          v-model="formData.ssh.enable"
                          color="primary"
                          hide-details
                          inset
                      ></v-switch>
                    </template>
                  </v-list-item>

                  <div v-show="formData.ssh.enable">
                    <div class="d-flex mt-2">
                      <div class="form-label">
                        {{ t('main.home.connector.form.host') }}
                      </div>
                      <div class="form-input">
                        <v-text-field
                            v-model="formData.ssh.host"
                            :rules="formRules.ssh.host"
                            density="comfortable"
                            :placeholder="t('main.home.connector.form.sshHostPlaceholder')"
                        ></v-text-field>
                      </div>
                    </div>

                    <div class="d-flex">
                      <div class="form-label">
                        {{ t('main.home.connector.form.port') }}
                      </div>
                      <div class="form-input">
                        <v-text-field
                            v-model="formData.ssh.port"
                            :rules="formRules.ssh.port"
                            type="number"
                            density="comfortable"
                            :placeholder="t('main.home.connector.form.sshPortPlaceholder')"
                        ></v-text-field>
                      </div>
                    </div>

                    <div class="d-flex">
                      <div class="form-label">
                        {{ t('common.user') }}
                      </div>
                      <div class="form-input">
                        <v-text-field
                            v-model="formData.ssh.user"
                            :rules="formRules.ssh.user"
                            density="comfortable"
                            :placeholder="t('main.home.connector.form.sshUserPlaceholder')"
                        ></v-text-field>
                      </div>
                    </div>

                    <div class="d-flex">
                      <div class="form-label form-radio-label">
                        {{ t('main.home.connector.form.identity') }}
                      </div>
                      <div class="form-input">
                        <v-radio-group v-model="formData.ssh.identity.model"
                                       inline
                                       hide-details
                        >
                          <v-radio
                              :label="t('common.none')"
                              value="none"
                          ></v-radio>
                          <v-radio
                              class="ml-2"
                              :label="t('common.password')"
                              value="password"
                          ></v-radio>
                          <v-radio
                              class="ml-2"
                              :label="t('common.privateKey')"
                              value="key"
                          ></v-radio>
                        </v-radio-group>

                        <div class="mt-4">
                          <v-expand-transition>
                            <div v-if="formData.ssh.identity.model == 'password'">
                              <v-text-field
                                  v-model="formData.ssh.identity.password"
                                  :rules="formRules.ssh.identity.password"
                                  :type="formPasswordShow.show2 ? 'text' : 'password'"
                                  :append-inner-icon="formPasswordShow.show2 ? 'mdi-eye-off' : 'mdi-eye'"
                                  @click:append-inner="formPasswordShow.show2 = !formPasswordShow.show2"
                                  density="comfortable"
                                  autocomplete
                                  :placeholder="t('main.home.connector.form.sshPasswordPlaceholder')"
                              ></v-text-field>
                            </div>
                            <div v-else-if="formData.ssh.identity.model == 'key'">
                              <SingleFileSelector
                                  v-model="formData.ssh.identity.key.key"
                                  :max-size="128*1024"
                                  :prompt-text="t('main.home.connector.form.sshKeyPlaceholder')"
                              ></SingleFileSelector>

                              <div class="mt-6 mb-2">
                                <p class="text-subtitle-2 text-medium-emphasis mb-2">
                                  {{ t('main.home.connector.form.rsaAlgorithm') }}</p>
                                <v-radio-group v-model="formData.ssh.identity.key.hashAlgorithm"
                                               inline
                                               hide-details
                                >
                                  <v-radio
                                      class="ml-0"
                                      :label="t('common.sha256')"
                                      value="sha256"
                                  />
                                  <v-radio
                                      class="ml-2"
                                      :label="t('common.sha512')"
                                      value="sha512"
                                  />
                                  <v-radio
                                      class="ml-2"
                                      :label="t('common.other')"
                                      value=""
                                  />
                                </v-radio-group>
                              </div>

                              <v-text-field
                                  class="mt-4"
                                  :label="t('common.password')"
                                  v-model="formData.ssh.identity.key.passphrase"
                                  :type="formPasswordShow.show3 ? 'text' : 'password'"
                                  :append-inner-icon="formPasswordShow.show3 ? 'mdi-eye-off' : 'mdi-eye'"
                                  @click:append-inner="formPasswordShow.show3 = !formPasswordShow.show3"
                                  density="comfortable"
                                  autocomplete
                                  :placeholder="t('main.home.connector.form.sshKeyPasswordPlaceholder')"
                              ></v-text-field>
                            </div>
                          </v-expand-transition>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </v-form>

            <div class="mt-8 pt-4 d-flex align-center justify-space-between flex-wrap"
                 style="gap: 16px; border-top: 1px solid rgba(var(--v-theme-on-surface), 0.12);">
              <v-btn class="text-capitalize text-none"
                     variant="text"
                     :ripple="false"
                     color="primary"
                     @click="testConnect"
              >
                <v-icon start>mdi-lan-pending</v-icon>
                {{ t('main.home.connector.form.testConnect') }}
              </v-btn>

              <div class="d-flex flex-grow-1 flex-sm-grow-0" style="gap: 12px;">
                <v-btn
                    variant="tonal"
                    color="primary"
                    class="flex-grow-1 flex-sm-grow-0 text-none px-6"
                    @click="saveConnection"
                >
                  <v-icon start>mdi-content-save</v-icon>
                  {{ t('common.save') }}
                </v-btn>
                <v-btn
                    color="primary"
                    class="flex-grow-1 flex-sm-grow-0 text-none px-8"
                    @click="connect"
                    elevation="2"
                >
                  <v-icon start>mdi-lan-connect</v-icon>
                  {{ t('common.connect') }}
                </v-btn>
              </div>
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
  $--form-label-width: 160px;

  .form-label {
    width: $--form-label-width;
    line-height: 52px;
    user-select: none;
  }

  .form-checkbox-label {
    line-height: 56px;
  }

  .form-radio-label {
    line-height: 40px;
  }

  .form-label:after {
    content: ":";
  }

  .form-input {
    width: calc(100% - $--form-label-width);
  }
}
</style>
