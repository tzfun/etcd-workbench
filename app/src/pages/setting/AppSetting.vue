<script setup lang="ts">
import Skeleton from "~/components/Skeleton.vue";
import {onMounted, reactive, ref, watch} from "vue";
import EditorExample from "~/components/editor/EditorExample.vue";
import {AppTheme} from "~/common/types.ts";
import {
  _alertError,
  _checkUpdateAndInstall,
  _confirmSystem,
  _emitGlobal,
  _tipSuccess,
  EventName
} from "~/common/events.ts";
import {DEFAULT_SETTING_CONFIG, SettingConfig} from "~/common/transport/setting.ts";
import {_debounce, _encodeStringToBytes, _goBrowserPage} from "~/common/utils.ts";
import WorkbenchLogo from "~/components/WorkbenchLogo.vue";
import {_loadAppVersion, _loadSettings, _setLocalSettings, _useSettings} from "~/common/store.ts";
import {appWindow} from "@tauri-apps/api/window";
import {useTheme} from "vuetify";
import {open, save} from "@tauri-apps/api/dialog";
import {_exportConnection, _handleError, _importConnection} from "~/common/services.ts";
import {_getDownloadPath, _isLinux, _isMac, _isWindows} from "~/common/windows.ts";

const theme = useTheme()

const groups = ['theme', 'connection', 'keys', 'update', 'about']
const activatedGroup = ref<string>('theme')
const editorTheme = reactive({
  light: [
    {
      label: 'Ayu Light',
      value: 'ayuLight'
    },
    {
      label: 'Clouds',
      value: 'clouds'
    },
    {
      label: 'Espresso',
      value: 'espresso',
    },
    {
      label: 'Noctis Lilac',
      value: 'noctisLilac',
    },
    {
      label: 'Rosé Pine Dawn',
      value: 'rosePineDawn',
    },
    {
      label: 'Smoothy',
      value: 'smoothy',
    },
    {
      label: 'Solarized Light',
      value: 'solarizedLight',
    },
    {
      label: 'Tomorrow',
      value: 'tomorrow'
    }
  ],
  dark: [
    {
      label: 'Amy',
      value: 'amy'
    },
    {
      label: 'Barf',
      value: 'barf'
    },
    {
      label: 'Bespin',
      value: 'bespin'
    },
    {
      label: 'Birds of Paradise',
      value: 'birdsOfParadise'
    },
    {
      label: 'Boys and Girls',
      value: 'boysAndGirls'
    },
    {
      label: 'Cobalt',
      value: 'cobalt'
    },
    {
      label: 'CoolGlow',
      value: 'coolGlow'
    },
    {
      label: 'Dracula',
      value: 'dracula'
    }
  ]
})

const exampleCode = `server:
  # Workbench running port
  port: 8002
  timeoutMillis: 3000
  dataPath: "./data"
auth:
  enable: false
  - user: user1:password1
  - user: user2:password2`
const exampleCodeLang = "yaml"

const settingForm = ref<SettingConfig>(JSON.parse(JSON.stringify(DEFAULT_SETTING_CONFIG)))
const appVersion = ref<string>('0.0.0')
const loadingStore = reactive({
  exportConnection: false,
  importConnection: false,
})

const connectionConfEncryptKeyRule = [
  (v?: string) => {
    let keyBytes = _encodeStringToBytes(v)
    if (keyBytes.length != 16) {
      return "Invalid byte length"
    }
    return true
  }
]

onMounted(async () => {
  await _loadSettings()
  settingForm.value = JSON.parse(JSON.stringify(_useSettings().value))
  appVersion.value = await _loadAppVersion()

  watch(() => settingForm.value, (v) => {
    let setting = {...v}
    if (typeof setting.kvLimitPerPage === 'string') {
      setting.kvLimitPerPage = parseInt(setting.kvLimitPerPage)
    }
    if (typeof setting.connectTimeoutSeconds === 'string') {
      setting.connectTimeoutSeconds = parseInt(setting.connectTimeoutSeconds)
    }
    if (typeof setting.requestTimeoutSeconds === 'string') {
      setting.requestTimeoutSeconds = parseInt(setting.requestTimeoutSeconds)
    }
    if (typeof setting.sshConnectTimeoutSeconds === 'string') {
      setting.sshConnectTimeoutSeconds = parseInt(setting.sshConnectTimeoutSeconds)
    }
    let keyBytes = _encodeStringToBytes(setting.connectionConfEncryptKey)
    if (keyBytes.length != 16) {
      return
    }
    _setLocalSettings(setting)
    _emitGlobal(EventName.SETTING_UPDATE, setting)
  }, {
    deep: true
  })

})

const setAppTheme = (appTheme: AppTheme) => {
  settingForm.value.theme = appTheme

  if (appTheme == 'auto') {
    appWindow.theme().then(systemTheme => {
      if (systemTheme) {
        theme.global.name.value = systemTheme
      }
    })
  } else {
    theme.global.name.value = appTheme
  }
}

const resetSettingConfig = () => {
  _confirmSystem('Are you sure you want to reset all settings?').then(() => {
    settingForm.value = JSON.parse(JSON.stringify(DEFAULT_SETTING_CONFIG))

    setAppTheme(settingForm.value.theme)

  }).catch(() => {
  })
}

const selectGroup = ({id}: any) => {
  let dom = document.getElementById(`setting-${id}`)
  if (dom) {
    dom.scrollIntoView({
      behavior: 'smooth',
      block: 'center'
    })
  }
}

const exportConnectionConfig = async () => {
  let downloadPath = await _getDownloadPath()
  save({
    filters: [{
      name: 'Etcd Workbench Config',
      extensions: ['wbc']
    }],
    defaultPath: downloadPath
  }).then(filepath => {
    if (filepath) {
      loadingStore.exportConnection = true
      _exportConnection(filepath).then(() => {
        _tipSuccess("Successfully exported")
      }).catch(e => {
        _handleError({e})
      }).finally(() => {
        loadingStore.exportConnection = false
      })
    }
  }).catch(e => {
    console.error(e)
    _alertError(e)
  })
}

const importConnectionConfig = () => {
  open({
    multiple: false,
    filters: [{
      name: 'Etcd Workbench Config',
      extensions: ['wbc']
    }]
  }).then(data => {
    if (data) {
      loadingStore.importConnection = true
      _importConnection(data as string).then(() => {
        _tipSuccess("Successfully imported")
        _emitGlobal(EventName.CONNECTION_IMPORTED)
      }).catch(e => {
        _handleError({
          e
        })
      }).finally(() => {
        loadingStore.importConnection = false
      })

    }
  }).catch(e => {
    console.error(e)
    _alertError(e)
  })
}

const onScroll = _debounce(() => {
  for (let group of groups) {
    let dom = document.getElementById(`setting-${group}`)
    if (dom) {
      let rect = dom.getBoundingClientRect()
      if (rect.top <= window.innerHeight && rect.bottom >= 0) {
        activatedGroup.value = group
        break
      }
    }
  }
}, 200)

</script>

<template>
  <v-sheet class="app-setting">
    <v-container class="fill-height pa-0" style="max-width: 1200px;">
      <v-layout class="fill-height overflow-y-auto position-relative">

        <v-navigation-drawer permanent class="user-select-none">
          <v-layout class="justify-center py-6">
            <WorkbenchLogo matrix font-size="20px"></WorkbenchLogo>
          </v-layout>
          <v-divider class="mb-5"></v-divider>
          <v-list lines="one"
                  activatable
                  :activated="activatedGroup"
                  mandatory
                  nav
                  density="compact"
                  @click:activate="selectGroup"
                  color="primary"
          >
            <v-list-item title="App Theme"
                         value="theme"
                         prepend-icon="mdi-brightness-6"
            ></v-list-item>
            <v-list-item title="Connection"
                         value="connection"
                         prepend-icon="mdi-transit-connection-variant"
            ></v-list-item>
            <v-list-item title="Keys"
                         value="keys"
                         prepend-icon="mdi-file-document-multiple"
            ></v-list-item>
            <v-list-item title="Update"
                         value="update"
                         prepend-icon="mdi-update"
            ></v-list-item>
            <v-list-item title="About"
                         value="about"
                         prepend-icon="mdi-information-variant-circle"
            ></v-list-item>
          </v-list>
        </v-navigation-drawer>
        <v-main class="overflow-y-auto" v-scroll.self="onScroll">
          <v-sheet class="pa-5">
            <v-layout>
              <v-spacer></v-spacer>
              <v-btn class="text-none"
                     color="red"
                     text="Reset All"
                     variant="elevated"
                     @click="resetSettingConfig"
              ></v-btn>
            </v-layout>

            <h3 class="group-title mt-5" id="setting-theme">App Theme</h3>
            <v-sheet class="d-flex mt-2 form-area">
              <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('light')">
                <Skeleton theme="light"
                          :active="settingForm.theme === 'light'"
                ></Skeleton>
                <p class="text-center text-medium-emphasis mt-2">Light</p>
              </div>
              <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('dark')">
                <Skeleton theme="dark"
                          :active="settingForm.theme === 'dark'"
                ></Skeleton>
                <p class="text-center text-medium-emphasis mt-2">Dark</p>
              </div>
              <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('auto')">
                <Skeleton theme="auto"
                          :active="settingForm.theme === 'auto'"
                ></Skeleton>
                <p class="text-center text-medium-emphasis mt-2">System</p>
              </div>
            </v-sheet>

            <h3 class="group-title mt-5" id="setting-connection">Connection</h3>
            <v-sheet class="mt-2 form-area pa-3">

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">File Encrypt Key</div>
                  <div class="v-messages">Local storage of connection profile encryption key.</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input" style="width: 200px;">
                  <v-text-field v-model="settingForm.connectionConfEncryptKey"
                                variant="outlined"
                                density="compact"
                                :counter="16"
                                persistent-counter
                                :rules="connectionConfEncryptKeyRule"
                  ></v-text-field>
                </div>
              </v-layout>
              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">Connect Timeout</div>
                  <div class="v-messages">Timeout for connecting to etcd server, in seconds.</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field v-model="settingForm.connectTimeoutSeconds"
                                variant="outlined"
                                type="number"
                                density="compact"
                                append-inner-icon="mdi-alpha-s"
                                hide-details
                  ></v-text-field>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">Request Timeout</div>
                  <div class="v-messages">Timeout for requesting etcd server, in seconds.</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field v-model="settingForm.requestTimeoutSeconds"
                                variant="outlined"
                                type="number"
                                density="compact"
                                append-inner-icon="mdi-alpha-s"
                                hide-details
                  ></v-text-field>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">SSH Connect Timeout</div>
                  <div class="v-messages">Timeout for connecting to ssh server, in seconds.</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field v-model="settingForm.sshConnectTimeoutSeconds"
                                variant="outlined"
                                type="number"
                                density="compact"
                                append-inner-icon="mdi-alpha-s"
                                hide-details
                  ></v-text-field>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">Close Tab By &nbsp;
                    <span class="text-cyan" v-if="_isWindows() || _isLinux()">Ctrl + W</span>
                    <span class="text-cyan" v-else-if="_isMac()">Command + W</span>
                  </div>
                  <div class="v-messages">
                    Use the
                    <i v-if="_isWindows() || _isLinux()">ctrl + w</i>
                    <i v-else-if="_isMac()">command + w</i>
                    shortcut key to close the current connection.
                  </div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch v-model="settingForm.closeTabUseCtrlW"
                            inset
                            density="compact"
                            color="primary"
                            hide-details
                            true-icon="mdi-check"
                  ></v-switch>
                </div>
              </v-layout>

              <v-layout class="pt-12 pb-5 justify-center align-center">
                <v-btn class="text-none"
                       text="Export Connections Configuration"
                       color="green-darken-3"
                       @click="exportConnectionConfig"
                       :loading="loadingStore.exportConnection"
                ></v-btn>

                <v-btn class="text-none ml-2"
                       text="Import Connections Configuration"
                       color="light-green-darken-1"
                       @click="importConnectionConfig"
                       :loading="loadingStore.importConnection"
                ></v-btn>
              </v-layout>
            </v-sheet>

            <h3 class="group-title mt-5" id="setting-keys">Keys</h3>
            <v-sheet class="mt-2 form-area pa-3">
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">Key Splitter</div>
                  <div class="v-messages">Parse the key path into a tree structure separator.</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field v-model="settingForm.kvPathSplitter"
                                variant="outlined"
                                density="compact"
                                hide-details
                  ></v-text-field>
                </div>
              </v-layout>
              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">Pagination Query</div>
                  <div class="v-messages">When the number of keys is large, you can enable paging query to optimize the
                    experience.
                  </div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch v-model="settingForm.kvPaginationQuery"
                            inset
                            density="compact"
                            color="primary"
                            hide-details
                            true-icon="mdi-check"
                  ></v-switch>
                </div>
              </v-layout>

              <div v-show="settingForm.kvPaginationQuery">
                <v-divider class="mt-5 mb-5"></v-divider>
                <v-layout>
                  <div>
                    <div class="form-label text-high-emphasis">Pagination Limit</div>
                    <div class="v-messages">Number of queries per page when querying all keys in pagination.</div>
                  </div>
                  <v-spacer></v-spacer>
                  <div class="form-input">
                    <v-text-field v-model="settingForm.kvLimitPerPage"
                                  type="number"
                                  variant="outlined"
                                  density="compact"
                                  hide-details
                    ></v-text-field>
                  </div>
                </v-layout>
              </div>

              <v-divider class="mt-5 mb-5"></v-divider>
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">Check Format Before Saving</div>
                  <div class="v-messages">Before saving the currently edited key each time, check whether the value
                    format is correct.
                  </div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch v-model="settingForm.kvCheckFormatBeforeSave"
                            inset
                            density="compact"
                            color="primary"
                            hide-details
                            true-icon="mdi-check"
                  ></v-switch>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <p class="mt-5 user-select-none">Editor Theme</p>
              <p class="v-messages">Set the Key-Value editor personalized theme.</p>
              <v-sheet class="mt-5 form-area">
                <v-sheet v-show="settingForm.theme === 'light' || settingForm.theme === 'auto'"
                         class="form-area">
                  <h4 class="text-center user-select-none">Light Theme</h4>
                  <v-row>
                    <v-col :cols="6"
                           v-for="(theme, idx) in editorTheme.light"
                           :key="idx"
                           class="editor-example mt-2"
                    >
                      <v-card hover :title="theme.label"
                              @click="settingForm.editorLightTheme = theme.value"
                              :ripple="false"
                              class="cursor-pointer"
                      >
                        <template #append>
                          <v-radio :value="theme.value"
                                   v-model="settingForm.editorLightTheme"
                                   color="primary"
                          ></v-radio>
                        </template>
                        <v-card-text>
                          <EditorExample
                              :content="exampleCode"
                              :theme="theme.value"
                              :content-language="exampleCodeLang"></EditorExample>
                        </v-card-text>
                      </v-card>

                    </v-col>
                  </v-row>
                </v-sheet>

                <v-sheet class="form-area mt-12" v-show="settingForm.theme === 'dark' || settingForm.theme === 'auto'">
                  <h4 class="text-center user-select-none">Dark Theme</h4>

                  <v-row>
                    <v-col :cols="6"
                           v-for="(theme, idx) in editorTheme.dark"
                           :key="idx"
                           class="editor-example mt-2"
                    >
                      <v-card hover :title="theme.label"
                              @click="settingForm.editorDarkTheme = theme.value"
                              :ripple="false"
                              class="cursor-pointer"
                      >
                        <template #append>
                          <v-radio :value="theme.value"
                                   v-model="settingForm.editorDarkTheme"
                                   color="primary"
                          ></v-radio>
                        </template>
                        <v-card-text>
                          <EditorExample
                              :content="exampleCode"
                              :theme="theme.value"
                              :content-language="exampleCodeLang"></EditorExample>
                        </v-card-text>
                      </v-card>
                    </v-col>
                  </v-row>
                </v-sheet>
              </v-sheet>

            </v-sheet>

            <h3 class="group-title mt-5" id="setting-update">Update</h3>
            <v-sheet class="mt-2 form-area pa-3">
              <v-layout>
                <div class="form-label text-high-emphasis">Version</div>
                <v-spacer></v-spacer>
                <div>
                  {{ appVersion }}
                </div>
              </v-layout>
              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">Update</div>
                <v-spacer></v-spacer>
                <div>
                  <v-btn class="text-none mr-2"
                         density="comfortable"
                         text="Check Update"
                         color="blue-lighten-1"
                         prepend-icon="mdi-arrow-up-bold-circle-outline"
                         @click="_checkUpdateAndInstall"
                  ></v-btn>
                  or
                  <v-btn class="text-none ml-2"
                         density="comfortable"
                         text="Download in GitHub"
                         prepend-icon="mdi-github"
                         color="grey-darken-4"
                         @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/releases/')"
                  >
                  </v-btn>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">Auto Update</div>
                  <div class="v-messages">Automatically update when a new version is available.</div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch v-model="settingForm.autoUpdate"
                            inset
                            density="compact"
                            color="primary"
                            hide-details
                            true-icon="mdi-check"
                  ></v-switch>
                </div>
              </v-layout>
            </v-sheet>

            <h3 class="group-title mt-5" id="setting-about">About</h3>
            <v-sheet class="mt-2 form-area pa-3">
              <div class="mb-12">
                <WorkbenchLogo class="my-5 cursor-pointer"
                               @click="_goBrowserPage('https://tzfun.github.io/etcd-workbench/')"
                               title="Etcd Workbench App"
                ></WorkbenchLogo>
                <p class="description my-3">A beautiful and lightweight gui client for ETCD V3</p>
                <p class="copyright">
                  Copyright &copy; 2024 <span class="link cursor-pointer"
                                              @click="_goBrowserPage('https://github.com/tzfun')">beifengtz</span>. All
                  rights reserved.
                </p>
              </div>

              <v-layout>
                <div class="form-label text-high-emphasis">Open Source</div>
                <v-spacer></v-spacer>
                <div>
                  <v-icon class="mr-2"
                          @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/')"
                          title="GitHub"
                  >mdi-github
                  </v-icon>
                  <v-icon @click="_goBrowserPage('https://gitee.com/tzfun/etcd-workbench/')"
                          title="Gitee"
                  >
                    <svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg" width="20" height="20">
                      <path
                          d="M512 1024C229.2224 1024 0 794.7776 0 512S229.2224 0 512 0s512 229.2224 512 512-229.2224 512-512 512z m259.1488-568.8832H480.4096a25.2928 25.2928 0 0 0-25.2928 25.2928l-0.0256 63.2064c0 13.952 11.3152 25.2928 25.2672 25.2928h177.024c13.9776 0 25.2928 11.3152 25.2928 25.2672v12.6464a75.8528 75.8528 0 0 1-75.8528 75.8528H366.592a25.2928 25.2928 0 0 1-25.2672-25.2928v-240.1792a75.8528 75.8528 0 0 1 75.8272-75.8528h353.9456a25.2928 25.2928 0 0 0 25.2672-25.2928l0.0768-63.2064a25.2928 25.2928 0 0 0-25.2672-25.2928H417.152a189.6192 189.6192 0 0 0-189.6192 189.6448v353.9456c0 13.9776 11.3152 25.2928 25.2928 25.2928h372.9408a170.6496 170.6496 0 0 0 170.6496-170.6496v-145.408a25.2928 25.2928 0 0 0-25.2928-25.2672z"
                          fill="#C71D23"></path>
                    </svg>
                  </v-icon>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">License</div>
                <v-spacer></v-spacer>
                <div>
                  <p class="text-blue cursor-pointer"
                     @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/blob/master/LICENSE')"
                     title="Click to view details"
                  >Apache License 2.0</p>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">Report Bug</div>
                <v-spacer></v-spacer>
                <v-btn class="text-none"
                       color="blue-grey-lighten-2"
                       variant="text"
                       @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/issues/new')"
                       title="Go to submit"
                       text="Submit"
                       prepend-icon="mdi-bug-outline"
                ></v-btn>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">Author</div>
                <v-spacer></v-spacer>
                <div>
                  <span class="text-medium-emphasis mr-2">beifengtz</span>
                  <v-icon class="mr-2"
                          @click="_goBrowserPage('mailto:beifengtz@qq.com')"
                          title="Email me"
                          color="blue"
                  >mdi-email-outline
                  </v-icon>

                  <v-icon class="mr-2"
                          @click="_goBrowserPage('https://github.com/tzfun/')"
                          title="Contact me on github"
                  >mdi-github
                  </v-icon>

                  <v-tooltip text="beifeng-tz"
                             location="top"
                  >
                    <template #activator="{ props }">
                      <v-icon class="mr-2"
                              v-bind="props"
                              title="Contact me on wechat"
                              color="green"
                      >mdi-wechat
                      </v-icon>
                    </template>

                  </v-tooltip>

                </div>
              </v-layout>
            </v-sheet>

          </v-sheet>
        </v-main>
      </v-layout>

    </v-container>
  </v-sheet>
</template>

<style scoped lang="scss">

.group-title {
  user-select: none;
  cursor: default;
}

.form-area {
  border-radius: 10px;
  margin: 8px 0;

  .form-label {
    user-select: none;
    display: flex;
    align-items: center;
  }

  .form-input {
    width: 120px;
  }
}

.app-theme-selected-icon {
  position: absolute;
  right: 2px;
  bottom: 32px;
  font-size: 60px;
}

.v-theme--light {
  .app-setting {
    background-color: #eeeeee;
    color: #000000;
  }

  .form-area {
    background-color: #f1ebeb;
  }
}

.v-theme--dark {
  .app-setting {
    background-color: #424242;
    color: #ffffff;
  }

  .form-area {
    background-color: #393838;
  }
}

.description {
  text-align: center;
  font-size: 20px;
}

.copyright {
  color: #9f9b9b;
  text-align: center;

  .link {
    color: unset;
    text-decoration-line: none;
    font-weight: 600;
  }

  .link:hover {
    text-decoration-line: underline;
  }
}
</style>
