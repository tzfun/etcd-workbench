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
import {_byteFormat, _sizeof} from "~/util/BaseUtil";

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

const allLanguages = reactive([
    'text',
    'json',
    'yaml',
    'xml'
])
const allTabSize = reactive([2,4,8])
const code = shallowRef(props.code)
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
  return _byteFormat(_sizeof(code.value))
})

const handleStateUpdate = (viewUpdate: ViewUpdate) => {
  // selected
  const ranges = viewUpdate.state.selection.ranges
  state.selected = ranges.reduce((plus, range) => plus + range.to - range.from, 0)
  state.cursor = ranges[0].anchor
  // length
  state.length = viewUpdate.state.doc.length
  state.lines = viewUpdate.state.doc.lines
  // log('viewUpdate', viewUpdate)
}

onMounted(() => {
  watch(
      () => props.code,
      (_code) => {
        code.value = _code
      }
  )
})

defineExpose({
  code: code
})
</script>
<template>
  <div>
    Language:
    <el-select v-model="config.language"
               fit-input-width
               style="width: 100px"
               class="m-2"
               placeholder="Select language">
      <el-option
          v-for="item in allLanguages"
          :key="item"
          :label="item"
          :value="item"
      />
    </el-select>
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
    <div class="editor">
      <div class="main">
        <codemirror
            v-model="code"
            :style="{
          width: '100%',
          height: config.height,
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
        />
      </div>
      <div class="divider"></div>
      <div class="footer">
        <div class="infos">
          <span class="item">Size: {{ size }}</span>
          <span class="item">Spaces: {{ config.tabSize }}</span>
          <span class="item">Length: {{ state.length }}</span>
          <span class="item">Lines: {{ state.lines }}</span>
          <span class="item">Cursor: {{ state.cursor }}</span>
          <span class="item">Selected: {{ state.selected }}</span>
        </div>
      </div>
    </div>
  </div>

</template>

<style lang="scss" scoped>
@import './variables.scss';

.editor {
  .divider {
    height: 1px;
    background-color: $border-color;
  }

  .main {
    display: flex;
    width: 100%;

    .code {
      width: 30%;
      height: 100px;
      margin: 0;
      padding: 0.4em;
      overflow: scroll;
      border-left: 1px solid $border-color;
      font-family: monospace;
    }
  }

  .footer {
    height: 3rem;
    padding: 0 1em;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 90%;

    .infos {
      .item {
        margin-left: 2em;
        display: inline-block;
        font-feature-settings: 'tnum';
      }
    }
  }
}
</style>
