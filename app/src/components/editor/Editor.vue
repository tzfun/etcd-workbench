<script setup lang="ts">

import {barf as darkTheme, espresso as lightTheme} from "./themes";
import {computed, onMounted, PropType, reactive, ref, shallowRef, watch} from "vue";
import {useTheme} from "vuetify";
import {EditorConfig} from "~/common/types.ts";
import jsonLanguage from "./lang/json";
import xmlLanguage from "./lang/xml";
import yamlLanguage from "./lang/yaml";
import sqlLanguage from "./lang/sql";
import propertiesLanguage from "./lang/properties";
import {EditorView} from "codemirror";
import {_byteTextFormat, _decodeBytesToString, _encodeStringToBytes, _strArrToNumArr} from "~/common/utils.ts";
import {Codemirror} from "vue-codemirror";

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
    required: true
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

const content = ref(props.value)

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

/**
 * 将当前内容读出为字符串，会根据选择的格式化语言进行转换
 */
const readDataString = (): string => {
  return formatData(content.value, "text", props.config!.language == 'blob' ? 'blob' : 'text')
}

defineExpose({
  readDataString
})

</script>

<template>
  <div class="fill-height">
    <div class="header">
      <slot name="headerPrepend"></slot>
      <div>
        <v-select
            variant="solo-filled"
            v-model="config.language"
            density="compact"
            :items="allLanguages"
            :width="140"
            hide-details
            persistent-hint
            class="mr-3"
        ></v-select>
      </div>
    </div>
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
      <slot name="footerPrepend"></slot>
      <span class="editor-footer-item"><strong>Size</strong>: {{ size }}</span>
    </div>
  </div>
</template>

<style scoped lang="scss">

$--editor-header-height: 50px;
$--editor-footer-height: 2rem;
$--editor-padding: 0 1rem;

.header {
  height: $--editor-header-height;
  display: flex;
  justify-content: right;
  align-items: center;
  font-size: 90%;

  .item {
    margin-left: 1em;
    display: inline-block;
    font-feature-settings: 'tnum';
  }
}

.editor {
  height: calc(100% - $--editor-header-height - $--editor-footer-height - 1px);

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
}
</style>
