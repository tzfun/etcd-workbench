<script setup lang="ts">
import {open, save} from "@tauri-apps/api/dialog";
import {listen} from "@tauri-apps/api/event";
import {appWindow} from "@tauri-apps/api/window";
import {onMounted, onUnmounted, reactive, ref, watch} from "vue";
import {useTheme} from "vuetify";
import {_alertError, _confirmSystem, _emitGlobal, _loading, _tipSuccess, EventName} from "~/common/events.ts";
import {_exportConnection, _handleError, _importConnection} from "~/common/services.ts";
import {_loadAppVersion, _loadSettings, _setLocalSettings, _useSettings} from "~/common/store.ts";
import {DEFAULT_SETTING_CONFIG, SettingConfig} from "~/common/transport/setting.ts";
import {AppTheme} from "~/common/types.ts";
import {_debounce, _encodeStringToBytes, _goBrowserPage} from "~/common/utils.ts";
import {_getDownloadPath, _isLinux, _isMac, _isWindows} from "~/common/windows.ts";
import EditorExample from "~/components/editor/EditorExample.vue";
import IconGitee from "~/components/icon/IconGitee.vue";
import IconPayPal from "~/components/icon/IconPayPal.vue";
import Skeleton from "~/components/Skeleton.vue";
import WorkbenchLogo from "~/components/WorkbenchLogo.vue";
import {_checkUpdate} from "~/common/updater.ts";
import {AllAppLanguages, AppLanguage} from "~/language";
import {useI18n} from "vue-i18n";

const theme = useTheme()
const {t, locale } = useI18n()

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
      return t('setting.encryptKeyLengthTip')
    }
    return true
  }
]

const eventUnListens = reactive<Function[]>([])

onMounted(async () => {
  await _loadSettings()
  settingForm.value = JSON.parse(JSON.stringify(_useSettings().value))
  appVersion.value = await _loadAppVersion()

  watch(() => settingForm.value, (v) => {
    let setting = { ...v }
    if (typeof setting.kvLimitPerPage === 'string') {
      setting.kvLimitPerPage = parseInt(setting.kvLimitPerPage)
    }
    if (typeof setting.kvSearchNextDirLimit === 'string') {
      setting.kvSearchNextDirLimit = parseInt(setting.kvSearchNextDirLimit)
    }
    if (typeof setting.kvDirRenameKeysLimit === 'string') {
      setting.kvDirRenameKeysLimit = parseInt(setting.kvDirRenameKeysLimit)
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
      console.debug("The length of 'connectionConfEncryptKey' must be 16", setting.connectionConfEncryptKey)
      return
    }
    if (setting.kvPathSplitter.length != 1) {
      console.debug("'kvPathSplitter' must be a single char",setting.kvPathSplitter)
      return;
    }
    _setLocalSettings(setting)
    _emitGlobal(EventName.SETTING_UPDATE, setting)
    setAppLanguage(setting.language)
  }, {
    deep: true
  })

  eventUnListens.push(await listen(EventName.SET_SETTING_ANCHOR, (e) => {
    const anchor = e.payload as string
    console.log(anchor)
  }))

})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
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

const setAppLanguage = (language: AppLanguage) => {
  locale.value = language
}

const resetSettingConfig = () => {
  _confirmSystem(t('setting.resetAllConfirm')).then(() => {
    settingForm.value = JSON.parse(JSON.stringify(DEFAULT_SETTING_CONFIG))

    setAppTheme(settingForm.value.theme)
    setAppLanguage(settingForm.value.language)
  }).catch(() => {
  })
}

const selectGroup = ({ id }: any) => {
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
        _tipSuccess(t('setting.exportConnectionTip'))
      }).catch(e => {
        _handleError({ e })
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
        _tipSuccess(t('setting.importConnectionTip'))
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

const checkUpdate = () => {
  _loading(true, t('setting.checkingUpdate'))
  _checkUpdate().then(available => {
    if (!available) {
      _tipSuccess(t('setting.alreadyLatestVersionTip'))
    }
  }).finally(()=> {
    _loading(false)
  })
}
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
          <v-list
              lines="one"
              activatable
              :activated="activatedGroup"
              mandatory
              nav
              density="compact"
              @click:activate="selectGroup"
              color="primary"
          >
            <v-list-item
                :title="t('setting.nav.app')"
                value="app"
                prepend-icon="mdi-brightness-6"
            />
            <v-list-item
                :title="t('setting.nav.connection')"
                value="connection"
                prepend-icon="mdi-transit-connection-variant"
            />
            <v-list-item
                :title="t('setting.nav.keys')"
                value="keys"
                prepend-icon="mdi-file-document-multiple"
            />
            <v-list-item
                :title="t('setting.nav.update')"
                value="update"
                prepend-icon="mdi-update"
            />
            <v-list-item
                :title="t('setting.nav.donate')"
                value="donate"
                prepend-icon="mdi-gift"
            />
            <v-list-item
                :title="t('setting.nav.about')"
                value="about"
                prepend-icon="mdi-information-variant-circle"
            />
          </v-list>
        </v-navigation-drawer>
        <v-main class="overflow-y-auto" v-scroll.self="onScroll">
          <v-sheet class="pa-5">
            <v-layout>
              <v-spacer></v-spacer>
              <v-btn
                  class="text-none"
                  color="red"
                  :text="t('setting.resetAll')"
                  variant="elevated"
                  @click="resetSettingConfig"
              />
            </v-layout>

            <h3 class="group-title mt-5" id="setting-app">{{ t('setting.nav.app')}}</h3>
            <v-sheet class="mt-2 form-area pa-3">

              <div class="text-high-emphasis">{{ t('setting.theme')}}</div>
              <v-layout>
                <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('light')">
                  <Skeleton theme="light" :active="settingForm.theme === 'light'"/>
                  <p class="text-center text-medium-emphasis mt-2">{{ t('setting.lightTheme')}}</p>
                </div>
                <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('dark')">
                  <Skeleton theme="dark" :active="settingForm.theme === 'dark'"></Skeleton>
                  <p class="text-center text-medium-emphasis mt-2">{{ t('setting.darkTheme')}}</p>
                </div>
                <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('auto')">
                  <Skeleton theme="auto" :active="settingForm.theme === 'auto'"></Skeleton>
                  <p class="text-center text-medium-emphasis mt-2">{{ t('setting.systemTheme')}}</p>
                </div>
              </v-layout>
              <v-divider class="mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.language') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div style="width: 200px;">
                  <v-select
                      v-model="settingForm.language"
                      :items="AllAppLanguages"
                      variant="outlined"
                      density="compact"
                      hide-details
                  />
                </div>
              </v-layout>

            </v-sheet>

            <h3 class="group-title mt-5" id="setting-connection">{{ t('setting.nav.connection')}}</h3>
            <v-sheet class="mt-2 form-area pa-3">

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.connectTimeout') }}</div>
                  <div class="v-messages">{{ t('setting.connectTimeoutDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field
                      v-model="settingForm.connectTimeoutSeconds"
                      variant="outlined"
                      type="number"
                      density="compact"
                      append-inner-icon="mdi-alpha-s"
                      hide-details
                  />
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.requestTimeout') }}</div>
                  <div class="v-messages">{{ t('setting.requestTimeoutDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field
                      v-model="settingForm.requestTimeoutSeconds"
                      variant="outlined"
                      type="number"
                      density="compact"
                      append-inner-icon="mdi-alpha-s"
                      hide-details
                  />
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.sshConnectTimeout') }}</div>
                  <div class="v-messages">{{ t('setting.sshConnectTimeoutDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field
                      v-model="settingForm.sshConnectTimeoutSeconds"
                      variant="outlined"
                      type="number"
                      density="compact"
                      append-inner-icon="mdi-alpha-s"
                      hide-details
                  />
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.closeTab') }}
                    <span class="text-cyan" v-if="_isWindows() || _isLinux()">Ctrl + W</span>
                    <span class="text-cyan" v-else-if="_isMac()">Command + W</span>
                  </div>
                  <div
                      class="v-messages"
                      v-html="t('setting.closeTabDesc', {shortcut: _isWindows() || _isLinux() ? 'ctrl + w' : 'command + w'})"
                  />
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch
                      v-model="settingForm.closeTabUseCtrlW"
                      inset
                      density="compact"
                      color="primary"
                      hide-details
                      true-icon="mdi-check"
                  />
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.fileEncryptKey') }}</div>
                  <div class="v-messages">{{ t('setting.fileEncryptKeyDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input" style="width: 200px;">
                  <v-text-field
                      v-model="settingForm.connectionConfEncryptKey"
                      variant="outlined"
                      density="compact"
                      :counter="16"
                      persistent-counter
                      :rules="connectionConfEncryptKeyRule"
                  />
                </div>
              </v-layout>
              
              <v-layout class="pt-12 pb-5 justify-center align-center">
                <v-btn
                    class="text-none"
                    :text="t('setting.exportConf')"
                    color="green-darken-3"
                    @click="exportConnectionConfig"
                    :loading="loadingStore.exportConnection"
                />

                <v-btn
                    class="text-none ml-2"
                    :text="t('setting.importConf')"
                    color="light-green-darken-1"
                    @click="importConnectionConfig"
                    :loading="loadingStore.importConnection"
                />
              </v-layout>
            </v-sheet>

            <h3 class="group-title mt-5" id="setting-keys">{{ t('setting.nav.keys') }}</h3>
            <v-sheet class="mt-2 form-area pa-3">
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.keySplitter') }}</div>
                  <div class="v-messages">{{ t('setting.keySplitterDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field
                      v-model="settingForm.kvPathSplitter"
                      variant="outlined"
                      density="compact"
                      hide-details
                  />
                </div>
              </v-layout>
              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.maxKeysForSearchSuggestions') }}</div>
                  <div class="v-messages">{{ t('setting.maxKeysForSearchSuggestionsDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field
                      v-model="settingForm.kvSearchNextDirLimit"
                      type="number"
                      variant="outlined"
                      density="compact"
                      hide-details
                  />
                </div>
              </v-layout>
              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.pathRenameKeyLimit') }}</div>
                  <div class="v-messages">{{ t('setting.pathRenameKeyLimitDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div class="form-input">
                  <v-text-field
                      v-model="settingForm.kvDirRenameKeysLimit"
                      type="number"
                      variant="outlined"
                      density="compact"
                      hide-details
                  />
                </div>
              </v-layout>
              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.paginationQuery') }}</div>
                  <div class="v-messages">{{ t('setting.paginationQueryDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch
                      v-model="settingForm.kvPaginationQuery"
                      inset
                      density="compact"
                      color="primary"
                      hide-details
                      true-icon="mdi-check"
                  />
                </div>
              </v-layout>

              <div v-show="settingForm.kvPaginationQuery">
                <v-divider class="mt-5 mb-5"></v-divider>
                <v-layout>
                  <div>
                    <div class="form-label text-high-emphasis">{{ t('setting.paginationLimit') }}</div>
                    <div class="v-messages">{{ t('setting.paginationLimitDesc') }}</div>
                  </div>
                  <v-spacer></v-spacer>
                  <div class="form-input">
                    <v-text-field
                        v-model="settingForm.kvLimitPerPage"
                        type="number"
                        variant="outlined"
                        density="compact"
                        hide-details
                    />
                  </div>
                </v-layout>
              </div>

              <v-divider class="mt-5 mb-5"></v-divider>
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.searchDirInTree') }}</div>
                  <div class="v-messages">{{ t('setting.searchDirInTreeDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch
                      v-model="settingForm.kvTreeSearchWithFolder"
                      inset
                      density="compact"
                      color="primary"
                      hide-details
                      true-icon="mdi-check"
                  />
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.checkFormatBeforeSave') }}</div>
                  <div class="v-messages">{{ t('setting.checkFormatBeforeSaveDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch
                      v-model="settingForm.kvCheckFormatBeforeSave"
                      inset
                      density="compact"
                      color="primary"
                      hide-details
                      true-icon="mdi-check"
                  />
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <p class="mt-5 user-select-none">{{ t('setting.editorTheme') }}</p>
              <p class="v-messages">{{ t('setting.editorThemeDesc') }}</p>
              <v-sheet class="mt-5 form-area">
                <v-sheet
                    v-show="settingForm.theme === 'light' || settingForm.theme === 'auto'"
                    class="form-area"
                >
                  <h4 class="text-center user-select-none">{{ t('setting.editorLightTheme') }}</h4>
                  <v-row>
                    <v-col
                        :cols="6"
                        v-for="(theme, idx) in editorTheme.light"
                        :key="idx"
                        class="editor-example mt-2"
                    >
                      <v-card
                          hover
                          :title="theme.label"
                          @click="settingForm.editorLightTheme = theme.value"
                          :ripple="false"
                          class="cursor-pointer"
                      >
                        <template #append>
                          <v-radio
                              :value="theme.value"
                              v-model="settingForm.editorLightTheme"
                              color="primary"
                          />
                        </template>
                        <v-card-text>
                          <EditorExample
                              :content="exampleCode"
                              :theme="theme.value"
                              :content-language="exampleCodeLang"
                          />
                        </v-card-text>
                      </v-card>

                    </v-col>
                  </v-row>
                </v-sheet>

                <v-sheet
                    class="form-area mt-12"
                    v-show="settingForm.theme === 'dark' || settingForm.theme === 'auto'"
                >
                  <h4 class="text-center user-select-none">{{ t('setting.editorDarkTheme') }}</h4>

                  <v-row>
                    <v-col
                        :cols="6"
                        v-for="(theme, idx) in editorTheme.dark"
                        :key="idx"
                        class="editor-example mt-2"
                    >
                      <v-card
                          hover
                          :title="theme.label"
                          @click="settingForm.editorDarkTheme = theme.value"
                          :ripple="false"
                          class="cursor-pointer"
                      >
                        <template #append>
                          <v-radio
                              :value="theme.value"
                              v-model="settingForm.editorDarkTheme"
                              color="primary"
                          />
                        </template>
                        <v-card-text>
                          <EditorExample
                              :content="exampleCode"
                              :theme="theme.value"
                              :content-language="exampleCodeLang"
                          />
                        </v-card-text>
                      </v-card>
                    </v-col>
                  </v-row>
                </v-sheet>
              </v-sheet>

            </v-sheet>

            <h3 class="group-title mt-5" id="setting-update">{{ t('setting.nav.update') }}</h3>
            <v-sheet class="mt-2 form-area pa-3">
              <v-layout>
                <div class="form-label text-high-emphasis">{{ t('setting.curVersion') }}</div>
                <v-spacer></v-spacer>
                <div>
                  {{ appVersion }}
                </div>
              </v-layout>
              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">{{ t('setting.nav.update') }}</div>
                <v-spacer></v-spacer>
                <div>
                  <v-btn
                      class="text-none mr-2"
                      density="comfortable"
                      :text="t('setting.checkUpdate')"
                      color="blue-lighten-1"
                      prepend-icon="mdi-arrow-up-bold-circle-outline"
                      @click="checkUpdate"
                  />
                  {{ t('common.or') }}
                  <v-btn
                      class="text-none ml-2"
                      density="comfortable"
                      :text="t('setting.downloadInGithub')"
                      prepend-icon="mdi-github"
                      color="grey-darken-4"
                      @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/releases/')">
                  </v-btn>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.autoUpdate') }}</div>
                  <div class="v-messages">{{ t('setting.autoUpdateDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-switch
                      v-model="settingForm.autoUpdate"
                      inset
                      density="compact"
                      color="primary"
                      hide-details
                      true-icon="mdi-check"
                  />
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>
              <v-layout>
                <div>
                  <div class="form-label text-high-emphasis">{{ t('setting.updateSource') }}</div>
                  <div class="v-messages">{{ t('setting.updateSourceDesc') }}</div>
                </div>
                <v-spacer></v-spacer>
                <div>
                  <v-radio-group v-model="settingForm.updateSource" inline>
                    <v-radio
                        label="GitHub"
                        value="github"
                        class="mr-6 font-weight-light"
                    />
                    <v-radio
                        label="Gitee"
                        value="gitee"
                    />
                  </v-radio-group>
                </div>
              </v-layout>
            </v-sheet>

            <h3 class="group-title mt-5" id="setting-donate">{{t('setting.nav.donate')}}</h3>

            <v-sheet class="mt-2 form-area pa-3">

              <div class="form-label text-high-emphasis mb-5">{{ t('setting.donateDesc') }}</div>
              
              <v-row class="my-8">
                <v-col
                    :cols="6"
                    class="donate-way"
                    :title="t('setting.donateViaPayPal')">
                  <IconPayPal
                      class="link cursor-pointer"
                      @click="_goBrowserPage('https://paypal.me/beifengtz')"
                  />
                </v-col>
                <v-col
                    :cols="6"
                    class="donate-way"
                    :title="t('setting.donateViaWechat')"
                >
                  <span style="position: relative;" 
                        class="link cursor-pointer donate-wechat-link">
                    <v-icon
                        :title="t('setting.supportAuthorCoffee')"
                        color="green">mdi-wechat</v-icon>
                      {{ t('setting.wechatRewards') }}
                      <img class="donate-wechat" src="/donate-wechat.jpg" alt="donate_wechat">
                    </span>
                </v-col>
              </v-row>
            </v-sheet>

            <h3 class="group-title mt-5" id="setting-about">{{ t('setting.nav.about') }}</h3>
            <v-sheet class="mt-2 form-area pa-3">
              <div class="mb-12">
                <WorkbenchLogo
                    class="my-5 cursor-pointer"
                    @click="_goBrowserPage('https://tzfun.github.io/etcd-workbench/')"
                    title="Etcd Workbench App"
                />
                <p class="description my-3"><i>{{ t('slogan') }}</i></p>
                <p class="copyright">
                  Copyright &copy; 2024 <span class="link cursor-pointer"
                    @click="_goBrowserPage('https://github.com/tzfun')">beifengtz</span>. All
                  rights reserved.
                </p>
              </div>

              <v-layout>
                <div class="form-label text-high-emphasis">{{t('setting.opensource')}}</div>
                <v-spacer></v-spacer>
                <div>
                  <v-icon
                      class="mr-2"
                      @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/')"
                      title="GitHub"
                  >mdi-github</v-icon>
                  <v-icon
                      @click="_goBrowserPage('https://gitee.com/tzfun/etcd-workbench/')"
                      title="Gitee">
                    <IconGitee></IconGitee>
                  </v-icon>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">{{t('setting.license')}}</div>
                <v-spacer></v-spacer>
                <div>
                  <p class="text-blue cursor-pointer"
                    @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/blob/master/LICENSE')"
                    :title="t('setting.clickToView')">GPL 3.0</p>
                </div>
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">{{t('setting.reportBug')}}</div>
                <v-spacer></v-spacer>
                <v-btn
                    class="text-none"
                    color="blue-grey-lighten-2"
                    variant="text"
                    @click="_goBrowserPage('https://github.com/tzfun/etcd-workbench/issues/new')"
                    :title="t('setting.goToSubmit')"
                    :text="t('common.submit')"
                    prepend-icon="mdi-bug-outline"
                />
              </v-layout>

              <v-divider class="mt-5 mb-5"></v-divider>

              <v-layout>
                <div class="form-label text-high-emphasis">{{ t('setting.author') }}</div>
                <v-spacer></v-spacer>
                <div>
                  <span class="text-medium-emphasis mr-2">beifengtz</span>
                  <v-icon
                      class="mr-2"
                      @click="_goBrowserPage('mailto:beifengtz@qq.com')"
                      :title="t('setting.emailMe')"
                      color="blue"
                  >mdi-email-outline</v-icon>

                  <v-icon
                      class="mr-2"
                      @click="_goBrowserPage('https://github.com/tzfun/')"
                      :title="t('setting.contactOnGithub')"
                  >mdi-github</v-icon>

                  <v-tooltip text="beifeng-tz" location="top">
                    <template #activator="{ props }">
                      <v-icon
                          class="mr-2"
                          v-bind="props"
                          :title="t('setting.contactOnWechat')"
                          color="green"
                      >mdi-wechat</v-icon>
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

.donate-way {
  justify-content: center;
  display: flex;
  align-self: center;
}

.donate-wechat-link {
  font-size: x-large;

  .donate-wechat {
    display: none;
    width: 200px;
    height: 240px;
    position: absolute;
    z-index: 1000;
    bottom: 50px;
    right: -30px;
    box-shadow: 5px 5px 10px rgba(125, 119, 119, 0.3215686275);
    border-radius: 15px;
  }
}

.donate-wechat-link:hover {
  .donate-wechat {
    display: block;
  }
}

</style>
