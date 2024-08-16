<script setup lang="ts">
import etcdLogo from "~/assets/etcd.png";
import Skeleton from "~/components/Skeleton.vue";
import {reactive, ref} from "vue";
import EditorExample from "~/components/editor/EditorExample.vue";
import {AppTheme} from "~/common/types.ts";
import {_emitGlobal} from "~/common/localEvents.ts";

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
  etcdExecuteTimeoutMillis: 3000
  dataPath: "./data"
auth:
  enable: false
  - user: user1:password1
  - user: user2:password2`
const exampleCodeLang = "yaml"

const appThemeForm = ref<AppTheme>('auto')
const editorThemeForm = reactive({
  dark: 'barf',
  light: 'smoothy'
})

const setAppTheme = (theme: AppTheme) => {
  appThemeForm.value = theme
  _emitGlobal('setAppTheme', theme)
}

</script>

<template>
  <v-sheet class="app-setting">
    <v-container class="fill-height pa-0" style="max-width: 1200px;">
      <v-layout class="fill-height overflow-y-auto  position-relative">

        <v-navigation-drawer permanent>
          <v-list-item class="ma-5"
          >
            <template #prepend>
              <v-img :src="etcdLogo"
                     cover
                     :width="30"
                     :height="30"
                     class="mr-2"
              ></v-img>
            </template>
            <template #title>
              <h3 class="text-center">Etcd Workbench</h3>
            </template>
          </v-list-item>
          <v-divider class="mb-5"></v-divider>
          <v-list lines="one"
                  activatable
                  activated="setting"
                  mandatory
                  nav
                  density="compact"
          >
            <v-list-item title="Setting"
                         value="setting"
                         prepend-icon="mdi-tune"
            ></v-list-item>
            <v-list-item title="Update"
                         value="update"
                         prepend-icon="mdi-update"
            ></v-list-item>
            <v-list-item title="About"
                         value="about"
                         prepend-icon="mdi-information-variant-circle-outline"
            ></v-list-item>
          </v-list>
        </v-navigation-drawer>
        <v-main class="overflow-y-auto">
          <v-sheet class=" pa-5">

            <h3>App Theme</h3>
            <v-sheet class="d-flex mt-5 form-area">
              <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('light')">

                <Skeleton theme="light"
                          :class="appThemeForm === 'light' ? 'app-theme-active' : ''"
                ></Skeleton>
                <p class="text-center text-medium-emphasis">Light</p>

                <v-icon v-show="appThemeForm === 'light'"
                        color="primary"
                        class="app-theme-selected-icon"
                >mdi-check-decagram-outline
                </v-icon>

              </div>
              <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('dark')">
                <Skeleton theme="dark"
                          :class="appThemeForm === 'dark' ? 'app-theme-active' : ''"
                ></Skeleton>
                <p class="text-center text-medium-emphasis">Dark</p>

                <v-icon v-show="appThemeForm === 'dark'"
                        color="primary"
                        class="app-theme-selected-icon"
                >mdi-check-decagram-outline
                </v-icon>

              </div>
              <div class="mx-auto my-5 cursor-pointer position-relative" @click="setAppTheme('auto')">
                <Skeleton theme="auto"
                          :class="appThemeForm === 'auto' ? 'app-theme-active' : ''"
                ></Skeleton>
                <p class="text-center text-medium-emphasis">System</p>

                <v-icon v-show="appThemeForm === 'auto'"
                        color="primary"
                        class="app-theme-selected-icon"
                >mdi-check-decagram-outline
                </v-icon>

              </div>
            </v-sheet>

            <h3 class="mt-5">Editor Theme</h3>
            <v-sheet class="mt-5">

              <v-sheet>
                <h4>Light</h4>
                <v-row class="form-area">
                  <v-col :cols="6"
                         v-for="(theme, idx) in editorTheme.light"
                         :key="idx"
                         class="editor-example mt-2"
                  >

                    <v-card hover :title="theme.label"
                            @click="editorThemeForm.light = theme.value"
                            :ripple="false"
                            class="cursor-pointer"
                    >
                      <template #append>
                        <v-radio :value="theme.value"
                                 v-model="editorThemeForm.light"
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

              <v-sheet class="mt-3">
                <h4>Dark</h4>

                <v-row class="form-area">
                  <v-col :cols="6"
                         v-for="(theme, idx) in editorTheme.dark"
                         :key="idx"
                         class="editor-example mt-2"
                  >
                    <v-card hover :title="theme.label"
                            @click="editorThemeForm.dark = theme.value"
                            :ripple="false"
                            class="cursor-pointer"
                    >
                      <template #append>
                        <v-radio :value="theme.value"
                                 v-model="editorThemeForm.dark"
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
        </v-main>
      </v-layout>

    </v-container>
  </v-sheet>
</template>

<style scoped lang="scss">

.form-area {
  border-radius: 10px;
  margin: 8px 0;
}

.app-theme-selected-icon {
  position: absolute;
  right: 0;
  bottom: 26px;
  font-size: 60px;
}

.v-theme--light {
  .app-setting {
    background-color: #eeeeee;
    color: #000000;
  }

  .form-area {
    background-color: #FAFAFA;
  }
}

.v-theme--dark {
  .app-setting {
    background-color: #424242;
    color: #ffffff;
  }

  .form-area {
    background-color: #5d5c5c;
  }
}
</style>
