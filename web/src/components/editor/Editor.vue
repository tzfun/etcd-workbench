<script lang="ts" setup>
import {EditorConfig} from "~/entitys/TransformTypes";
import {computed, onMounted, reactive, shallowRef, watch} from "vue";
import {oneDark} from "@codemirror/theme-one-dark";
import {EditorView, ViewUpdate} from "@codemirror/view";
import {redo, undo} from "@codemirror/commands";
import {Codemirror} from "vue-codemirror";
import jsonLanguage from "./lang/json";
import xmlLanguage from "./lang/xml";
import yamlLanguage from "./lang/yaml";
import {_byteFormat, _bytesToStr, _hexToStr, _strToBytes, _strToHex} from "~/util/Util";

const props = defineProps({
  config: {
    type: Object as EditorConfig,
    required: true
  },
  code: {
    type: String,
    required: true
  }
})

const emits = defineEmits(["change","save"])

const allLanguages = reactive([
  'text',
  'hex',
  'blob',
  'json',
  'yaml',
  'xml'
])

/**
 * 格式化数据
 * @param content 数据内容
 * @param newLang 新的格式语言
 * @param curLang 当前的数据格式语言
 */
const formatData = (content, newLang, curLang) => {
  if (newLang === curLang) {
    return content
  }
  let contentStr = content
  if (curLang == "hex") {
    contentStr = _hexToStr(contentStr.replace(/\\x/g, ""))
  } else if (curLang == "blob") {
    contentStr = _bytesToStr(contentStr.split(" "))
  }

  if (newLang === "hex") {
    contentStr = _strToHex(contentStr)
  } else if (newLang === "blob") {
    contentStr = _strToBytes(contentStr).join(" ")
  }

  return contentStr
}

const allTabSize = reactive([2, 4, 8])
const code = shallowRef(formatData(props.code, props.config.language, 'text'))
const extensions = computed(() => {
  const result = []
  switch (props.config.language) {
    case 'json':
      result.push(jsonLanguage())
      break
    case 'yml':
      result.push(xmlLanguage())
      break
    case 'yaml':
      result.push(yamlLanguage())
      break
  }

  result.push(props.config.theme !== 'default' ? oneDark : void 0)
  return result
})

const cmView = shallowRef<EditorView>()
const handleReady = ({view}: any) => {
  cmView.value = view
}
const onChanged = (data) => {
  emits('change', data)
}

const onKeyDown = (event: KeyboardEvent) => {
  if (event.key == 's' && (event.ctrlKey || event.metaKey)) {
    event.preventDefault()
    emits('save')
  }
}

// https://github.com/codemirror/commands/blob/main/test/test-history.ts
const handleUndo = () => {
  undo({
    state: cmView.value!.state,
    dispatch: cmView.value!.dispatch
  })
}

const handleRedo = () => {
  redo({
    state: cmView.value!.state,
    dispatch: cmView.value!.dispatch
  })
}

const state = reactive({
  lines: null as null | number,
  cursor: null as null | number,
  selected: null as null | number,
  length: null as null | number
})

const size = computed(() => {
  return _byteFormat(formatData(code.value, 'blob', props.config.language).length)
})

const handleStateUpdate = (viewUpdate: ViewUpdate) => {
  const ranges = viewUpdate.state.selection.ranges
  state.selected = ranges.reduce((plus, range) => plus + range.to - range.from, 0)
  state.cursor = ranges[0].anchor
  state.length = viewUpdate.state.doc.length
  state.lines = viewUpdate.state.doc.lines
}

onMounted(() => {
  watch(
      () => props.code,
      (_code) => {
        code.value = formatData(_code, props.config.language, 'text')
      }
  )

  watch(
      () => props.config.language,
      (lang, preLang) => {
        code.value = formatData(code.value, lang, preLang)
      }
  )
})

const upperFirst = (s: string): string => {
  return s.substring(0, 1).toUpperCase() + s.substring(1)
}

/**
 * 将当前内容读出为字符串，会根据选择的格式化语言进行转换
 */
const readDataString = (): string => {
  return formatData(code.value, "text", props.config.language)
}

defineExpose({
  readDataString
})
</script>
<template>
  <div style="height: 100%">
    <div class="header">
      <slot name="headerAppender"></slot>
      <div class="item">
        Format:
        <el-select v-model="config.language"
                   fit-input-width
                   style="width: 100px"
                   class="m-2"
                   placeholder="Select language">
          <el-option
              v-for="item in allLanguages"
              :key="item"
              :label="upperFirst(item)"
              :value="item"
          />
        </el-select>
      </div>
      <div class="item">
        Tab Size:
        <el-select v-model="config.tabSize"
                   fit-input-width
                   style="width: 80px"
                   class="m-2"
                   placeholder="Select tab size">
          <el-option
              v-for="item in allTabSize"
              :key="item"
              :label="item"
              :value="item"
          />
        </el-select>
      </div>
    </div>
    <div class="editor">
      <codemirror
          v-model="code"
          :style="{
          width: '100%',
          height: config.height,
          'font-size': config.fontSize,
          backgroundColor: '#fff',
          color: '#333'
        }"
          placeholder="Please enter the content."
          :extensions="extensions"
          :autofocus="config.autofocus"
          :disabled="config.disabled"
          :indent-with-tab="config.indentWithTab"
          :tab-size="config.tabSize"
          @update="handleStateUpdate"
          @ready="handleReady"
          @change="onChanged"
          @keydown="onKeyDown"
      />
      <div class="divider"></div>
      <div class="footer">
        <slot name="footerAppender"></slot>
        <div class="infos">
          <span class="item">Size: {{ size }}</span>
<!--          <span class="item">Spaces: {{ config.tabSize }}</span>-->
          <span class="item">Length: {{ state.length }}</span>
          <span class="item">Lines: {{ state.lines }}</span>
<!--          <span class="item">Cursor: {{ state.cursor }}</span>-->
<!--          <span class="item">Selected: {{ state.selected }}</span>-->
        </div>
      </div>
    </div>
  </div>

</template>

<style lang="scss" scoped>

$--editor-header-height: 3rem;
$--editor-footer-height: 3rem;
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
  height: calc(100% - $--editor-header-height - $--editor-footer-height);

  .divider {
    height: 1px;
    background-color: var(--theme-border);
  }

  .code {
    width: 30%;
    height: 100px;
    margin: 0;
    padding: 0.4em;
    overflow: scroll;
    border-left: 1px solid var(--theme-border);
    font-family: monospace;
  }

  .footer {
    height: $--editor-footer-height;
    padding: $--editor-padding;
    display: flex;
    justify-content: right;
    align-items: center;
    font-size: 90%;

    .infos {
      .item {
        margin-left: 1em;
        display: inline-block;
        font-feature-settings: 'tnum';
      }
    }
  }
}
</style>
