<script setup lang="ts">
import {Delete, Document, DocumentCopy, Finished, Folder, InfoFilled, Search, Tickets} from "@element-plus/icons-vue";
import {EditorConfig, KeyValueDTO, TreeNode} from "~/common/Types";
import {reactive, ref} from "vue";
import {_parseCodeLanguage} from "~/common/Util";
import WorkbenchLogo from "~/design/WorkbenchLogo.vue";

const EMPTY_KV = {
  key: '',
  value: '',
  version: 0,
  createRevision: 0,
  modRevision: 0,
  lease: 0
}
const props = defineProps({
  data: Array<TreeNode>,
})
const emits = defineEmits(['on-select', 'on-save', 'on-delete', 'on-diff', 'copy-and-save'])

const keySearch = ref()
const treeRef = ref()
const editorRef = ref()
const currentNode = ref<TreeNode>()
const changed = ref<Boolean>()
const treeDefaultProps = {
  children: 'children',
  label: 'label'
}
const editingKV = ref<KeyValueDTO>(EMPTY_KV)

const editorConfig = reactive<EditorConfig>({
  disabled: false,
  indentWithTab: true,
  tabSize: 2,
  autofocus: false,
  height: "100%",
  fontSize: "1rem",
  language: 'json'
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

const editorSave = () => {
  if (changed.value) {
    saveKV()
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
          editorConfig.language = _parseCodeLanguage(node.label, data.value)
        }
      })
    }
  } else {
    let tmp = node
    while (tmp.children && tmp.children.length == 1) {
      tmp = tmp.children[0]
    }
    treeRef.value.setCurrentKey(tmp.path)
  }
}

const saveKV = () => {
  const key = currentNode.value!.path
  const value = editorRef.value.readDataString()

  emits('on-save', {
    kv: {
      key: key,
      value: value
    },
    callback: () => {
      changed.value = false
    }
  })
}

const diff = () => {
  emits('on-diff', currentNode.value?.data)
}

const copyAndSave = () => {
  emits('copy-and-save', currentNode.value?.path)
}

const del = () => {
  const key = currentNode.value?.path
  emits('on-delete', {
    key: key,
    callback: () => {
      clear()
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

const clear = () => {
  clearSelectedKeys()
  editingKV.value = EMPTY_KV
  currentNode.value = undefined
}

defineExpose({
  getSelectedKeys,
  clearSelectedKeys,
  clear
})

</script>

<template>
  <div class="tree-container">
    <div class="tree-aside" ref="asideRef">
      <el-input v-model="keySearch" placeholder="Type to search" :prefix-icon="Search" class="search-input"/>
      <div class="tree">
        <el-tree
            ref="treeRef"
            class="flow-tree"
            :data="data"
            highlight-current
            show-checkbox
            node-key="path"
            :props="treeDefaultProps"
            :filter-node-method="filterTreeNode"
            @node-click="clickTreeNode"
        >
          <template #default="{ node, data }">
            <span class="tree-node-icon">
              <el-icon v-if="data.type === 'dir'" class="node-icon-folder"><Folder/></el-icon>
              <el-icon v-else class="node-icon-doc"><Document/></el-icon>
            </span>
            <span :class="data.type === 'file' ? 'tree-node-file' : 'tree-node-dir'">{{ node.label }}</span>
          </template>
        </el-tree>
      </div>
    </div>
    <div class="tree-editor">
      <editor ref="editorRef"
              v-if="currentNode"
              :key="editingKV"
              :code="(editingKV as KeyValueDTO).value"
              :config="editorConfig"
              @change="editorChange"
              @save="editorSave">
        <template #headerAppender>
          <div>
            <el-button type="primary" :icon="Tickets" size="small" @click="saveKV">Save{{
                changed ? " *" : ""
              }}
            </el-button>
            <el-button type="info" :icon="DocumentCopy" size="small" @click="diff">Version Diff</el-button>
            <el-button type="warning" :icon="Finished" size="small" @click="copyAndSave">Copy And Save</el-button>
            <el-button type="danger" :icon="Delete" size="small" @click="del">Delete</el-button>
          </div>
        </template>
        <template #footerAppender>
          <div>
            <span class="item">{{ currentNode.path }}</span>
            <span class="item"><strong>Version</strong>: {{ editingKV.version }}</span>
            <span class="item"><strong>Create Revision</strong>: {{ editingKV.createRevision }}</span>
            <span class="item"><strong>Modify Revision</strong>: {{ editingKV.modRevision }}</span>
            <span class="item"><strong>Lease</strong>: {{ editingKV.lease }}</span>
          </div>
        </template>
      </editor>
      <div v-else class="no-key-preview">
        <workbench-logo matrix logo-size="150px" font-size="80px"/>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
@import '../../styles/index.scss';

.tree-container {
  width: 100%;
  display: flex;

  $--tree-aside-width: 300px;
  $--tree-aside-padding-right: 15px;

  .tree-aside {
    width: $--tree-aside-width;
    border-right: solid 1px var(--ep-menu-border-color);
    padding-right: $--tree-aside-padding-right;
    position: relative;

    .search-input {
      height: 30px;
    }

    .tree {
      height: calc(100% - 30px);
      overflow-y: auto;

      .tree-node-icon {
        margin: 0 5px;
        line-height: 0;

        .node-icon-folder {
          color: #949393;
        }

        .node-icon-doc {
          color: #ae57f0;
        }
      }
    }
  }

  .tree-editor {
    width: calc(100% - $--tree-aside-width - $--tree-aside-padding-right);
    height: 100%;

    .item {
      margin-left: 1em;
      display: inline-block;
      font-feature-settings: 'tnum';
    }

    .node-key {
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .no-key-preview {
      width: 100%;
      height: 100%;
      overflow: hidden;
      display: flex;
      justify-content: center;
      align-items: center;

      filter: grayscale(30%);
      opacity: 0.5;
    }
  }
}

//.tree-node-file {
//  color: #2e9f52;
//}

.flow-tree {
  min-width: 100%;
  display: inline-block;
}
</style>
