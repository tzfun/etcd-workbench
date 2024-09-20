<script setup lang="ts">

import {computed, onMounted, onUnmounted, PropType, reactive, ref, shallowRef, watch} from "vue";
import {useTheme} from "vuetify";
import {EditorConfig} from "~/common/types.ts";
import jsonLanguage from "./lang/json";
import xmlLanguage from "./lang/xml";
import yamlLanguage from "./lang/yaml";
import sqlLanguage from "./lang/sql";
import propertiesLanguage from "./lang/properties";
import {EditorView} from "codemirror";
import {
  _byteTextFormat,
  _decodeBytesToString,
  _encodeStringToBytes, _pointInRect,
  _strArrToNumArr,
  _upperCaseFirst,
  fileTypeIcon
} from "~/common/utils.ts";
import {Codemirror} from "vue-codemirror";
import {appWindow} from "@tauri-apps/api/window";
import {_useSettings} from "~/common/store.ts";
import {getThemeByName} from "~/components/editor/themes.ts";
import {VSheet} from "vuetify/components";
import {platform as getPlatform} from "@tauri-apps/api/os";

import * as prettier from "prettier/standalone";
import prettierPluginBabel from "prettier/plugins/babel";
import prettierPluginHtml from "prettier/plugins/html";
import prettierPluginYaml from "prettier/plugins/yaml";
import prettierPluginEstree from "prettier/plugins/estree";
import prettierPluginSql from "prettier-plugin-sql";

type ContentFormatType = 'text' | 'blob'

const props = defineProps({
  config: {
    type: Object as PropType<EditorConfig>,
    required: true
  },
  value: {
    type: String,
    default: () => "",
  }
})

const emits = defineEmits(["change", "save"])

const enabledFormatLanguage = new Set(["json", "yaml", "xml", "sql"])

const allLanguages = reactive([
  'text',
  'blob',
  'json',
  'yaml',
  'xml',
  'sql',
  'properties'
])
const showLanguageSelection = ref<boolean>(false)
const consolePanelData = reactive({
  show: false,
  content: ""
})
const languageSelectionBoxRef = ref()

const content = ref<String>(props.value)
const propsConfig = ref(props.config!)

const tauriBlurUnListen = ref<Function>()
const platform = ref<'win32' | 'darwin' | string>('win32')


onMounted(async () => {
  platform.value = await getPlatform()

  tauriBlurUnListen.value = await appWindow.listen('tauri://blur', () => {
    showLanguageSelection.value = false
  })

  document.addEventListener('mousedown', (e: MouseEvent) => {
    if (showLanguageSelection.value) {
      if (languageSelectionBoxRef.value) {
        let rect = ((languageSelectionBoxRef.value as VSheet).$el as HTMLElement).getBoundingClientRect()
        if (rect) {
          if (!_pointInRect(e, rect)) {
            showLanguageSelection.value = false
          }
        }
      }
    }
  })

  document.addEventListener('keydown', (e: KeyboardEvent) => {
    if (platform.value == 'darwin') {
      if (e.metaKey && e.altKey && e.key == 'l') {
        formatContent()
      }
    } else if (platform.value == 'win32') {
      if (e.ctrlKey && e.altKey && e.key == 'l') {
        formatContent()
      }
    }

  })
})

onUnmounted(() => {
  if (tauriBlurUnListen.value) {
    tauriBlurUnListen.value()
  }
})

/**
 * 格式化数据
 *
 * @param content     数据内容
 * @param fromFormat  当前的数据格式，只能为 text 和 blob
 * @param toFormat    新的数据格式
 */
const formatData = (content: string, fromFormat: ContentFormatType, toFormat: ContentFormatType): string => {
  if (toFormat === fromFormat) {
    return content
  }

  //  blob to text
  if (fromFormat == 'blob') {
    let uint8Array = _strArrToNumArr(content.trim().split(/\s+/))
    return _decodeBytesToString(uint8Array)
  }

  //  text to blob
  if (toFormat == 'blob') {
    let uint8Array = _encodeStringToBytes(content)
    let newContent = ""
    const SPLIT_LEN = 20
    for (let i = 0; i < uint8Array.length; i += SPLIT_LEN) {
      let end = Math.min(i + SPLIT_LEN, uint8Array.length)
      for (let j = i; j < end; j++) {
        newContent += `${uint8Array[j]} `
      }
      newContent += '\n'
    }

    return newContent
  }

  return content
}

const extensions = computed(() => {
  const result = []
  switch (props.config.language) {
    case 'json':
      result.push(jsonLanguage())
      break
    case 'xml':
      result.push(xmlLanguage())
      break
    case 'yaml':
      result.push(yamlLanguage())
      break
    case 'sql':
      result.push(sqlLanguage())
      break
    case 'properties':
      result.push(propertiesLanguage())
      break
  }

  let appTheme = useTheme().global.name.value
  let setting = _useSettings().value;
  let themeName
  if (appTheme == 'dark') {
    themeName = setting.editorDarkTheme
  } else {
    themeName = setting.editorLightTheme
  }

  result.push(getThemeByName(themeName))
  return result
})
const cmView = shallowRef<EditorView>()
const size = computed(() => {
  return _byteTextFormat(_encodeStringToBytes(content.value).length)
})

watch(
    () => props.value,
    (_code: string) => {
      content.value = formatData(_code, 'text', props.config!.language == 'blob' ? 'blob' : 'text')
    }
)

watch(
    () => props.config!.language,
    (newLang, oldLang) => {
      if (newLang != oldLang) {
        content.value = formatData(
            content.value,
            oldLang == 'blob' ? 'blob' : 'text',
            newLang == 'blob' ? 'blob' : 'text',
        )
      }
    }, {
      deep: true
    }
)

const handleReady = ({view}: any) => {
  const cm = view as EditorView
  cmView.value = cm
  //  scroll to top
  cm.dispatch({
    selection: {
      anchor: 0,
      head: 0
    },
    scrollIntoView: true
  });
}

const onChanged = (data: string) => {
  emits('change', data)
}

const onKeyDown = (event: KeyboardEvent) => {
  if (event.key == 's' && (event.ctrlKey || event.metaKey)) {
    event.preventDefault()
    emits('save')
  }
}

const changeLanguage = (lang: string) => {
  propsConfig.value.language = lang
  showLanguageSelection.value = false
}

//  对当前内容进行格式化
const formatContent = () => {
  showLanguageSelection.value = false
  let language = props.config?.language
  if (!enabledFormatLanguage.has(language)) {
    return
  }
  let parser
  let plugins = []
  switch (language) {
    case 'json':
      parser = 'json-stringify';
      plugins.push(prettierPluginBabel)
      plugins.push(prettierPluginEstree)
      break
    case 'xml':
      parser = 'html';
      plugins.push(prettierPluginHtml)
      break
    case 'yaml':
      parser = 'yaml'
      plugins.push(prettierPluginYaml)
      break
    case 'sql':
      parser = 'sql'
      plugins.push(prettierPluginSql)
          break
  }
  prettier.format(content.value, {
    parser: parser,
    plugins: plugins,
    bracketSameLine: true
  }).then(newContent => {
    consolePanelData.show = false
    let oldContent = content.value
    content.value = newContent
    if (newContent != oldContent) {
      onChanged(newContent)
    }
  }).catch(e => {
    openConsolePanel(e.toString())
  })
}

const openConsolePanel = (content: string) => {
  consolePanelData.content = content
  consolePanelData.show = true
}

/**
 * 将当前内容读出为 byte 数组
 */
const readDataBytes = (): number[] => {
  if (propsConfig.value.language == 'blob') {
    return _strArrToNumArr(content.value.trim().split(/\s+/))
  } else {
    return _encodeStringToBytes(content.value)
  }
}

defineExpose({
  readDataBytes
})

</script>

<template>
  <div class="fill-height position-relative border-solid border-sm border-opacity">
    <div class="editor">
      <codemirror
          v-model="content"
          style="width: 100%;height: 100%;"
          :extensions="extensions"
          :autofocus="config.autofocus"
          :disabled="config.disabled"
          :indent-with-tab="config.indentWithTab"
          :tab-size="config.tabSize"
          @ready="handleReady"
          @change="onChanged"
          @keydown="onKeyDown"
      />
    </div>
    <v-divider></v-divider>
    <div class="footer">
      <slot name="footer"></slot>
      <span class="editor-footer-item"><strong>Size</strong>: {{ size }}</span>
      <span class="editor-footer-item">
        <span class="text-primary cursor-pointer user-select-none"
              @click="showLanguageSelection = !showLanguageSelection"
        >
          <v-icon>{{ fileTypeIcon[config.language] }}</v-icon>
          {{ _upperCaseFirst(config.language) }}
        </span>
      </span>

      <v-sheet class="editor-language-selection card-box-shadow"
               v-show="showLanguageSelection"
               ref="languageSelectionBoxRef"
      >
        <v-list density="compact"
        >
          <v-list-item v-for="item in allLanguages"
                       :key="item"
                       :value="item"
                       :title="_upperCaseFirst(item)"
                       :active="item == config.language"
                       :prepend-icon="fileTypeIcon[item]"
                       @click="changeLanguage(item)"
                       color="primary"
          ></v-list-item>
        </v-list>
        <div v-if="enabledFormatLanguage.has(config.language)">
          <v-divider></v-divider>
          <v-list density="compact"
          >
            <v-list-item title="Format"
                         color="primary"
                         @click="formatContent"
                         class="text-center"
            >
              <template #title>
                Format
                <span class="text-medium-emphasis" v-if="platform == 'win32'">
                  (
                  <span class="font-weight-bold" style="font-size: 0.9em">Ctrl</span> +
                  <span class="font-weight-bold" style="font-size: 0.9em">Alt</span> +
                  <span class="font-weight-bold" style="font-size: 0.9em">L</span>
                  )
                </span>
                <span class="text-medium-emphasis" v-else-if="platform == 'darwin'">
                  (
                  <v-icon size="0.9em" class="font-weight-bold">mdi-apple-keyboard-command</v-icon> +
                  <v-icon size="0.9em" class="font-weight-bold">mdi-apple-keyboard-option</v-icon> +
                  <span class="font-weight-bold">L</span>
                  )
                </span>
              </template>
            </v-list-item>
          </v-list>
        </div>
      </v-sheet>

      <v-sheet class="console-panel border-t-md"
               v-show="consolePanelData.show"
      >
        <v-icon class="console-panel-close"
                @click="consolePanelData.show = false"
        >mdi-close
        </v-icon>
        <pre><code>{{ consolePanelData.content }}</code></pre>
      </v-sheet>
    </div>
  </div>
</template>

<style scoped lang="scss">
@import "~/styles/variables";

$--editor-padding: 0 1rem;

.editor {
  height: calc(100% - $--editor-footer-height - 2px);

  .code {
    width: 30%;
    height: 100px;
    margin: 0;
    padding: 0.4em;
    overflow: scroll;
    border-left: 1px solid var(--theme-border);
    font-family: monospace;
  }
}

.footer {
  height: $--editor-footer-height;
  padding: $--editor-padding;
  display: flex;
  justify-content: right;
  align-items: center;
  font-size: 90%;
  overflow: auto;

  .editor-language-selection {
    position: absolute;
    width: 200px;
    min-height: 200px;
    z-index: 100;
    bottom: $--editor-footer-height;
    border: 1px solid rgba(90, 90, 90, 0.12);
    color-scheme: normal;
  }

  .console-panel {
    position: absolute;
    width: 100%;
    height: 300px;
    z-index: 10;
    bottom: $--editor-footer-height;
    left: 0;
    padding: 15px;
    overflow: auto;
    font-size: 1em;

    .console-panel-close {
      position: absolute;
      right: 15px;
      top: 15px;
    }
  }
}
</style>
