<script setup lang="ts">

import {barf as darkTheme, smoothy as lightTheme} from "./themes";
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
  _encodeStringToBytes,
  _strArrToNumArr,
  _upperCaseFirst,
  fileTypeIcon
} from "~/common/utils.ts";
import {Codemirror} from "vue-codemirror";
import {appWindow} from "@tauri-apps/api/window";

type ContentFormatType = 'text' | 'blob'

const theme = useTheme()

const editorTheme = ref<any>(darkTheme)

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


const content = ref(props.value)
const propsConfig = ref(props.config!)

const tauriBlurUnListen = ref<Function>()

onMounted(async () => {
  tauriBlurUnListen.value = await appWindow.listen('tauri://blur', () => {
    showLanguageSelection.value = false
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
    console.log('blob to text', content.split(/\s+/), uint8Array, content)
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
    console.log('text to blob', content, newContent)

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
  result.push(editorTheme.value)
  return result
})
const cmView = shallowRef<EditorView>()
const size = computed(() => {
  return _byteTextFormat(_encodeStringToBytes(content.value).length)
})

watch(
    () => theme,
    (newTheme) => {
      if (newTheme.global.name.value == 'dark') {
        editorTheme.value = darkTheme
      } else {
        editorTheme.value = lightTheme
      }
    }, {
      deep: true
    }
)

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

onMounted(() => {
  if (theme.global.name.value == 'dark') {
    editorTheme.value = darkTheme
  } else {
    editorTheme.value = lightTheme
  }
})

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

const onfocusout = (e) => {
  console.log(e)
}

</script>

<template>
  <div class="fill-height position-relative border-solid border-sm border-opacity">
    <div class="editor">
      <codemirror
          v-model="content"
          style="width: 100%;height: 100%;"
          placeholder="Please enter the content."
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

      <div class="editor-language-selection card-box-shadow"
           v-show="showLanguageSelection"
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
                       onfocusout="onfocusout"
          ></v-list-item>
        </v-list>
      </div>
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
}
</style>
