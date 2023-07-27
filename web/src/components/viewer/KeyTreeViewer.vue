<script setup lang="ts">
import {Delete, DocumentCopy, Edit, Search, Tickets} from "@element-plus/icons-vue";
import {EditorConfig, KeyDTO, KeyValueDTO, TreeNode} from "~/entitys/TransformTypes";
import {reactive} from "vue";
import {isDark} from "~/composables";

const props = defineProps({
  data: Array<TreeNode>,
})
const emits = defineEmits(['on-select', 'on-save', 'on-delete', 'on-diff'])

const keySearch = ref()
const treeRef = ref()
const editorRef = ref()
const currentNode = ref<TreeNode>()
const changed = ref<Boolean>()
const treeDefaultProps = {
  children: 'children',
  label: 'label'
}
const editingKV = ref<KeyValueDTO>({
  key: '',
  value: ''
})

const editorConfig = reactive<EditorConfig>({
  disabled: false,
  indentWithTab: true,
  tabSize: 2,
  autofocus: false,
  height: "100%",
  fontSize: "1.2rem",
  language: 'json',
  theme: isDark ? 'oneDark' : 'default'
})

watch(keySearch, (val) => {
  if (treeRef.value) {
    treeRef.value.filter(val)
  }
})

const filterTreeNode = (value: string, data: TreeNode) => {
  if (!value) return true
  return data.path.includes(value)
}

const editorChange = () => {
  if (currentNode.value) {
    changed.value = true
  }
}

const clickTreeNode = (node: TreeNode) => {
  if (node.type === "file") {
    if (currentNode.value != node) {
      currentNode.value = node
      emits('on-select', {
        key: node.path,
        callback: data => {
          changed.value = false
          editingKV.value = data
          const content = data.value
          if (content.startsWith('<')) {
            editorConfig.language = 'xml'
          } else if (content.startsWith('{') || content.startsWith('[')) {
            editorConfig.language = 'json'
          } else if (content.startsWith('---')) {
            editorConfig.language = 'yaml'
          } else {
            editorConfig.language = 'text'
          }
        }
      })
    }
  }
}

const saveKV = () => {
  const key = currentNode.value?.path
  const value = editorRef.value.readDataString()

  emits('on-save', {
    key: key,
    value: value,
    callback: () => {
      changed.value = false
    }
  })
}

const diff = () => {
  emits('on-diff', currentNode.value?.data)
}

const del = () => {
  const key = currentNode.value?.path
  emits('on-delete', {
    key: key,
    callback: () => {
      currentNode.value = null
      changed.value = false
    }
  })
}

const getSelectedKeys = (): string[] => {
  const selectedKeys = []
  for (let key of treeRef.value.getCheckedKeys(false)) {
    if (!key.startsWith("@")) {
      selectedKeys.push(key)
    }
  }
  return selectedKeys
}

const clearSelectedKeys = () => {
  treeRef.value!.setCheckedKeys([], false)
}

defineExpose({
  getSelectedKeys,
  clearSelectedKeys
})

</script>

<template>
  <div class="tree-container">
    <div class="tree-aside" ref="asideRef">
      <el-input v-model="keySearch" placeholder="Type to search" :prefix-icon="Search"/>
      <el-tree
          ref="treeRef"
          :data="data"
          highlight-current
          show-checkbox
          node-key="path"
          :props="treeDefaultProps"
          :filter-node-method="filterTreeNode"
          @node-click="clickTreeNode"
      />
    </div>
    <div class="tree-editor">
      <editor ref="editorRef"
              :key="editingKV"
              :code="(editingKV as KeyValueDTO).value"
              :config="editorConfig"
              @change="editorChange">
        <template #headerAppender>
          <div v-if="currentNode">
            <el-button type="primary" :icon="Tickets" plain size="small" @click="saveKV">Save{{ changed ? " *" : "" }}</el-button>
            <el-button type="info" :icon="DocumentCopy" plain size="small" @click="diff">Version Diff</el-button>
            <el-button type="danger" :icon="Delete" size="small" @click="del">Delete</el-button>
          </div>
        </template>
        <template #footerAppender>
          <div v-if="currentNode" class="editor-footer">
            <span class="item"><strong>Version</strong>: {{ currentNode.data.version }}</span>
            <span class="item"><strong>Create Revision</strong>: {{ currentNode.data.createRevision }}</span>
            <span class="item"><strong>Modify Revision</strong>: {{ currentNode.data.modRevision }}</span>
          </div>
        </template>
      </editor>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@import '../../styles/index.scss';

.tree-container {
  width: 100%;
  display: flex;

  $--tree-aside-width: 400px;
  $--tree-aside-padding-right: 15px;

  .tree-aside {
    width: $--tree-aside-width;
    border-right: solid 1px var(--ep-menu-border-color);
    padding-right: $--tree-aside-padding-right;;
  }

  .tree-editor {
    width: calc(100% - $--tree-aside-width - $--tree-aside-padding-right);
    height: 100%;

    .editor-footer {
      .item {
        margin-left: 2em;
        display: inline-block;
        font-feature-settings: 'tnum';
      }
    }
  }
}
</style>