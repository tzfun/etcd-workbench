<script setup lang="ts">
import {computed, onMounted, onUnmounted, PropType, reactive, ref} from "vue";

import '@ztree/ztree_v3/js/jquery-1.4.4.min';
import '@ztree/ztree_v3/js/jquery.ztree.core.js';
import '@ztree/ztree_v3/js/jquery.ztree.excheck.js';
import '@ztree/ztree_v3/js/jquery.ztree.exhide.js';
import '@ztree/ztree_v3/js/jquery.ztree.exedit.js';
//  @ts-ignore
import {fuzzySearch} from './ztree-fuzzysearch'
import {SessionData} from "~/common/transport/connection.ts";
import {_useSettings} from "~/common/store.ts";
import {VCard} from "vuetify/components";
import {appWindow} from "@tauri-apps/api/window";
import {KeyExtendInfo} from "~/common/transport/kv.ts";
import {TreeNode} from "~/components/tree/types.ts";
import {useLocale} from "vuetify";

const IDMark_A = "_a"

export type ContextmenuKeyword =
    'delete'
    | 'rename'
    | 'addToMonitor'
    | 'editMonitor'
    | 'addToCollection'
    | 'removeFromCollection'
    | 'versionDiff'
    | 'addKey'
    | 'copyAndSave'

export type ContextmenuItem = {
  title: string,
  keyword: ContextmenuKeyword,
  icon?: string,
  iconColor?: string,
  baseColor?: string,
}

export type ContextmenuExtend = {
  type: 'divider'
}

export type Contextmenu = ContextmenuItem | ContextmenuExtend

const {t} = useLocale()
const appSettings = _useSettings()
const emits = defineEmits(['on-click', 'on-click-remove', 'click:contextmenu'])
const props = defineProps({
  treeId: {
    type: String,
    required: true
  },
  keySplitter: {
    type: String,
    default: "/"
  },
  enableSearch: {
    type: Boolean,
    default: () => true
  },
  session: {
    type: Object as PropType<SessionData>,
    required: true
  },
  showCheckBox: {
    type: Boolean,
    default: () => true
  },
  initItems: {
    type: Array<string>
  },
  showNodeSuffix: {
    type: Boolean,
    default: () => true
  },
  showHoverRemove: {
    type: Boolean,
    default: () => false
  },
  enableSelect: {
    type: Boolean,
    default: () => true
  },
  enableContextmenu: {
    type: Boolean,
    default: () => false
  }
})
const keyId = computed<string>(() => {
  return `key-${props.treeId}`
})
const treeRootObj = ref()
const triggerRemovedKey = ref<string>()
const expandState = ref<boolean>(false)
const contextmenuRef = ref<VCard>()
const contextmenuStorage = reactive({
  treeId: "",
  treeNode: <null | TreeNode>null,
})
const contextmenu = ref<Contextmenu[]>([])
const tauriBlurUnListen = ref<Function>()

onMounted(async () => {
  if (props.enableContextmenu) {
    document.addEventListener('click', () => {
      setTimeout(() => {
        if (contextmenuRef.value) {
          const element: Element = contextmenuRef.value.$el
          element.removeAttribute('style')
        }
      }, 30)
    })

    tauriBlurUnListen.value = await appWindow.listen('tauri://blur', () => {
      if (contextmenuRef.value) {
        const element: Element = contextmenuRef.value.$el
        element.removeAttribute('style')
      }
    })
  }
})

onUnmounted(() => {
  if (tauriBlurUnListen.value) {
    tauriBlurUnListen.value()
  }
})

const beforeClick = (_treeId: string, treeNode: TreeNode) => {
  if (!treeNode) {
    return false;
  }
  if (treeNode.isParent) {
    treeRootObj.value.expandNode(treeNode)
    return false;
  }
  return treeNode.id != triggerRemovedKey.value;
}

const onClick = (_e: MouseEvent, _treeId: string, treeNode: TreeNode) => {
  if (!treeNode.isParent) {
    emits('on-click', treeNode.id, treeNode.keyInfo)
  }
  if (!props.enableSelect) {
    treeRootObj.value.cancelSelectedNode(treeNode)
  }
}

const onRightClick = (e: MouseEvent, treeId: string, treeNode: TreeNode) => {
  contextmenuStorage.treeId = treeId
  contextmenuStorage.treeNode = treeNode
  if (contextmenuRef.value) {
    const element: Element = contextmenuRef.value.$el
    if (treeNode) {
      let menuHeight = 10
      const menu: Contextmenu[] = []

      function pushItem(item: ContextmenuItem) {
        menu.push(item)
        menuHeight += 32
      }

      function pushExtend(extend: ContextmenuExtend) {
        menu.push(extend)
        if (extend.type === 'divider') {
          //  margin: 8px 0
          //  height: 1px
          menuHeight += 17
        }
      }

      pushItem({
        title: 'Rename',
        keyword: 'rename',
        icon: 'mdi-rename',
      })

      if (treeNode.isParent) {
        pushItem({
          title: 'Add Key',
          keyword: 'addKey',
          icon: 'mdi-file-document-plus-outline',
          iconColor: 'green'
        })
      } else {
        pushItem({
          title: 'Copy and Save',
          keyword: 'copyAndSave',
          icon: 'mdi-content-copy',
          iconColor: 'light-green-darken-1'
        })
        pushItem({
          title: 'Version Diff',
          keyword: 'versionDiff',
          icon: 'mdi-vector-difference',
          iconColor: 'cyan-darken-1'
        })
      }

      pushExtend({
        type: 'divider'
      })

      //  位于monitor列表中
      if (props.session!.keyMonitorMap![treeNode.id]) {
        pushItem({
          title: 'Edit monitor',
          keyword: 'editMonitor',
          icon: 'mdi-robot',
          iconColor: '#cc8f53',
        })
      } else {
        pushItem({
          title: 'Add to monitors',
          keyword: 'addToMonitor',
          icon: 'mdi-robot-outline',
          iconColor: '#cc8f53',
        })
      }

      //  文件节点
      if (!treeNode.isParent) {
        //  位于收藏列表中
        if (props.session!.keyCollectionSet!.has(treeNode.id)) {
          pushItem({
            title: 'Remove from collections',
            keyword: 'removeFromCollection',
            icon: 'mdi-star',
            iconColor: '#ced10a',
          })
        } else {
          pushItem({
            title: 'Add to collections',
            keyword: 'addToCollection',
            icon: 'mdi-star-outline',
            iconColor: '#ced10a',
          })
        }
      }

      pushExtend({
        type: 'divider'
      })
      pushItem({
        title: 'Delete',
        keyword: 'delete',
        icon: 'mdi-trash-can-outline',
        baseColor: 'red'
      })

      let left = e.clientX
      let top = e.clientY
      //  下边界保护
      const windowHeight = window.innerHeight
      if (top + menuHeight >= windowHeight) {
        //  20px边距
        top = windowHeight - menuHeight - 20
      }

      contextmenu.value = menu
      const style = `left: ${left}px; top: ${top}px; display:unset;`
      element.setAttribute("style", style)
    } else {
      //  右击空白处
      element.removeAttribute('style')
    }
  }
}

const showTitle = (_treeId: string, node: TreeNode) => {
  return !node.isParent
}

const addDiyDom = (_treeId: string, node: TreeNode) => {
  if (!node) {
    return
  }
  diyDom(node, false)
}

const diyDom = (node: TreeNode, refresh: boolean) => {
  if (props.showNodeSuffix) {
    //  刷新，动态判断已有的按钮是否应该存在
    if (refresh) {
      let aObj
      //  @ts-ignore
      let starDom = $(`#${node.tId}${IDMark_A} .icon-star-filled`)

      if (!node.isParent) {
        //  只作用于文件
        if (props.session!.keyCollectionSet!.has(node.id)) {
          if (!starDom || starDom.length == 0) {
            //  @ts-ignore
            aObj = $("#" + node.tId + IDMark_A)
            let star = `<span class="icon-star-filled tree-node-icon" onfocus='this.blur();'></span>`
            aObj.append(star)
          }
        } else {
          if (starDom) {
            starDom.remove()
          }
        }
      }

      //  @ts-ignore
      let monitorDom = $(`#${node.tId}${IDMark_A} .icon-monitor`)
      //  即可以作用于目录也可以作用于文件
      if (props.session!.keyMonitorMap![node.id]) {
        if (!monitorDom || monitorDom.length == 0) {
          if (!aObj) {
            //  @ts-ignore
            aObj = $("#" + node.tId + IDMark_A)
          }
          let monitor = `<span class="icon-monitor tree-node-icon" onfocus='this.blur();'></span>`
          aObj.append(monitor)
        }
      } else {
        if (monitorDom) {
          monitorDom.remove()
        }
      }
    } else {
      //  只添加
      let aObj
      if (!node.isParent && props.session!.keyCollectionSet!.has(node.id)) {
        //  @ts-ignore
        aObj = $("#" + node.tId + IDMark_A)
        let star = `<span class="icon-star-filled tree-node-icon" onfocus='this.blur();'></span>`
        aObj.append(star)
      }

      if (props.session!.keyMonitorMap![node.id]) {
        if (!aObj) {
          //  @ts-ignore
          aObj = $("#" + node.tId + IDMark_A)
        }
        let monitor = `<span class="icon-monitor tree-node-icon" onfocus='this.blur();'></span>`
        aObj.append(monitor)
      }
    }
  }
}

const addHoverDom = (_treeId: string, node: TreeNode) => {
  if (!node || node.isParent) {
    return
  }

  if (props.showHoverRemove) {
    //  @ts-ignore
    if ($(`#removeBtn_${node.tId}`).length > 0) {
      return;
    }

    //  @ts-ignore
    let aObj = $("#" + node.tId + IDMark_A);
    let star = `<span class="icon-remove tree-node-icon" id="removeBtn_${node.tId}" onfocus='this.blur();' title="Remove"></span>`
    aObj.append(star)
    //  @ts-ignore
    let btn = $(`#removeBtn_${node.tId}`)
    if (btn) {
      let key = node.id
      btn.bind('click', function () {
        triggerRemovedKey.value = key
        emits('on-click-remove', key)
      })
    }
  }
}

const removeHoverDom = (_treeId: string, node: TreeNode) => {
  if (!node || node.isParent) {
    return
  }
  if (props.showHoverRemove) {
    //  @ts-ignore
    $(`#removeBtn_${node.tId}`).unbind().remove()
  }
}

const settings = {
  data: {
    key: {
      title: "id"
    },
    simpleData: {
      enable: true
    }
  },
  view: {
    nameIsHTML: true, //  允许name支持html
    nodeClasses: {add: ['tree-item']},
    showLine: false,
    dblClickExpand: false,
    selectedMulti: false,
    showTitle: showTitle,
    addHoverDom: addHoverDom,
    removeHoverDom: removeHoverDom,
    addDiyDom: addDiyDom,
  },
  callback: {
    beforeClick: beforeClick,
    onClick: onClick,
    onRightClick: props.enableContextmenu ? onRightClick : undefined
  },
  check: {
    enable: true
  },
  edit: {
    enable: false,
    editNameSelectAll: false
  }
}

onMounted(() => {
  settings.check.enable = props.showCheckBox
  rerender()
  if (props.initItems) {
    props.initItems.forEach(key => {
      addItemToTree(key)
    })
  }
})

/**
 * 重新渲染树结构，并初始化数据
 */
const rerender = () => {
  let tree = treeRootObj.value
  if (tree) {
    //  @ts-ignore
    $.fn.zTree.destroy(props.treeId);
  }
  //  @ts-ignore
  treeRootObj.value = $.fn.zTree.init($(`#${props.treeId}`), settings, [])

  fuzzySearch(
      props.treeId,
      `#${keyId.value}`,
      null,
      false,
      appSettings
  )
}

/**
 * 根据ID获取树中的节点信息
 * @param id 树的id
 * @return {TreeNode | undefined}
 */
const getTreeNodeById = (id: any): TreeNode | undefined => {
  return treeRootObj.value.getNodesByParam("id", id, null)[0]
}

const addItemToTree = (key: string, ignoreIfExist?: boolean, keyInfo?: KeyExtendInfo) => {
  if (ignoreIfExist) {
    let node = getTreeNodeById(key)
    if (node) {
      return
    }
  }
  let id
  if (key.startsWith(props.keySplitter)) {
    id = props.keySplitter
  } else {
    //  为了方便解析为统一的树状结构，如果key不是以分隔符开头，默认补充分隔符
    key = props.keySplitter + key
    id = ""
  }

  let dirs = key.split(props.keySplitter)
  let fileName = dirs[dirs.length - 1]

  let parentNode = null
  //  遍历并创建目录
  for (let i = 1; i < dirs.length - 1; i++) {
    let dirName = dirs[i]
    id += `${dirName}${props.keySplitter}`

    let node = getTreeNodeById(id)
    //  节点不存在，添加节点
    if (!node) {
      let newNode: TreeNode = constructDirNode(id, dirName, parentNode ? parentNode.id : undefined)
      treeRootObj.value.addNodes(parentNode, newNode, true)

      parentNode = getTreeNodeById(id)
    } else {
      parentNode = node
    }
  }

  id += fileName
  let newNode: TreeNode = constructFileNode(id, fileName, parentNode ? parentNode.id : undefined, keyInfo)
  treeRootObj.value.addNodes(parentNode, newNode, true)
}

const removeItemFromTree = (id: string) => {
  let node = getTreeNodeById(id)
  if (node) {
    treeRootObj.value.removeNode(node)
    //  移除之后如果父节点是空的，继续移除父节点
    while (true) {
      if (node.pId) {
        let parentNode = getTreeNodeById(node.pId)
        if (parentNode && (!parentNode.children || parentNode.children.length == 0)) {
          treeRootObj.value.removeNode(parentNode)
          node = parentNode
        } else {
          break
        }
      } else {
        break
      }
    }
  }
}

const constructDirNode = (id: string, name: string, pId: string | undefined): TreeNode => {
  return {
    id: id,
    pId: pId,
    name: name,
    isParent: true,
    open: false,
    iconOpen: "/folder-open.png",
    iconClose: "/folder.png",
  }
}

const constructFileNode = (id: string, name: string, pId: string | undefined, keyInfo?: KeyExtendInfo): TreeNode => {
  return {
    id,
    pId,
    name,
    isParent: false,
    open: false,
    icon: "/file-text.png",
    keyInfo,
  }
}

const getSelectedItems = (): TreeNode[] => {
  return treeRootObj.value.getCheckedNodes(true) || []
}

const refreshDiyDom = (key: string) => {
  let node = getTreeNodeById(key)
  if (node) {
    diyDom(node, true)
  }
}

const cancelSelected = () => {
  treeRootObj.value.cancelSelectedNode()
}

const selectItem = (key: string) => {
  let node = getTreeNodeById(key)
  if (node) {
    treeRootObj.value.selectNode(node)
  }
}
/**
 * 展开/折叠全部
 * @param expand 状态。true-展开，false-折叠
 * @return boolean 状态。true-展开，false-折叠
 */
const expandAll = (expand: boolean): boolean => {
  return treeRootObj.value.expandAll(expand)
}

const toggleExpand = () => {
  expandState.value = !expandState.value
  expandAll(expandState.value)
}

const onclickContextmenu = (item: ContextmenuItem) => {
  emits('click:contextmenu', item.keyword, contextmenuStorage.treeNode)
}

defineExpose({
  addItemToTree,
  removeItemFromTree,
  rerender,
  getSelectedItems,
  refreshDiyDom,
  cancelSelected,
  selectItem,
  expandAll
})

</script>

<template>
  <div>
    <div v-if="enableSearch" class="position-relative">
      <v-tooltip location="top"
                 no-click-animation
                 text="To enable/disable directory search, go to settings.">
        <template v-slot:activator="{ props }">
          <v-icon class="search-icon"
                  v-bind="props"
          >mdi-magnify
          </v-icon>
        </template>
      </v-tooltip>
      <input
          type="text"
          :id="keyId"
          value=""
          class="search-input"
          :placeholder="t('common.typeToSearch')"
      />

      <v-btn class="expand-icon"
             icon="mdi-arrow-expand-vertical"
             title="Expand or collapse all"
             size="x-small"
             variant="plain"
             @click="toggleExpand"
      ></v-btn>
    </div>
    <div :id="treeId" class="ztree key-tree overflow-auto px-1"
         :style="`height:${enableSearch ? 'calc(100% - 46px)' : '100%'};`"></div>
    <v-card
        ref="contextmenuRef"
        elevation="16"
        border
        class="contextmenu pa-0"
        density="compact"
    >
      <v-list
          density="compact"
          color="light-blue"
          lines="one"
      >
        <div v-for="(item,i) in contextmenu" :key="i">
          <v-divider class="my-2" v-if="(item as ContextmenuExtend).type == 'divider'"/>
          <v-list-item
              v-else
              density="compact"
              @click="onclickContextmenu(item as ContextmenuItem)"
              :title="(item as ContextmenuItem).title"
              :base-color="(item as ContextmenuItem).baseColor"
          >
            <template #prepend v-if="(item as ContextmenuItem).icon">
              <v-icon :color="(item as ContextmenuItem).iconColor">{{ (item as ContextmenuItem).icon }}</v-icon>
            </template>
          </v-list-item>
        </div>
      </v-list>
    </v-card>
  </div>
</template>

<style lang="scss">
$--search-input-x-margin: 8px;
$--search-white-border-color: #9da4a8;
$--search-black-border-color: #4c4d4f;

$--search-hover-border-color: #6c6e72;
$--search-focus-border-color: rgb(33, 150, 243);

$--expand-icon-width: 32px;
$--expand-icon-margin: 3px;

.search-icon {
  $--fixed-margin: 5px;
  position: absolute;
  left: calc($--search-input-x-margin + $--fixed-margin);
  top: calc($--search-input-x-margin + $--fixed-margin);
}

.expand-icon {
  width: $--expand-icon-width;
  margin-left: $--expand-icon-margin;
  margin-right: $--expand-icon-margin;
}

.search-input {
  padding-left: 30px;
  padding-right: 30px;
  border: 1px solid;
  border-radius: 3px;
  font-size: 0.9em;
  width: calc(100% - $--search-input-x-margin - $--expand-icon-width - $--expand-icon-margin * 2);
  height: 30px;
  margin: $--search-input-x-margin 0 $--search-input-x-margin $--search-input-x-margin;
  background-color: rgba(0, 0, 0, 0);
  transition: all ease 0.2s;
  outline: none;
}

.search-input:hover,
.search-input:focus {
  border-color: $--search-hover-border-color !important;
}

.tree-item {
  text-underline: none;
  text-decoration: none;
}

.ztree {
  * {
    font-size: 1em !important;
    font-family: unset !important;
  }
}

.key-tree {
  .tree-item {
    user-select: none;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  $--tree-item-height: 30px;

  .roots_docu:before,
  .roots_docu:after,
  .center_docu:before,
  .bottom_docu:before,
  .center_docu:after,
  .bottom_docu:after {
    position: absolute;
    content: "";
    border: 0 dotted #dbdbdb;
  }

  .button {
    position: relative;
    display: inline-block;
    line-height: 22px;
    height: 22px;
    width: 22px;
    cursor: pointer;
    text-align: center;
    vertical-align: middle;
  }

  .button.chk {
    position: relative;
    width: 14px;
    height: 14px;
    margin: 0 8px 0 0;
    border: 1px solid rgba(215, 221, 228, 0.7);
    border-radius: 2px;
    background: rgba(0, 0, 0, 0);
  }

  .button.chk:after {
    position: absolute;
    top: 1px;
    left: 4px;
    width: 4px;
    height: 8px;
    content: "";
    transition: 0.2s ease-in-out;
    -webkit-transition: 0.2s ease-in-out;
    -ms-transition: 0.2s ease-in-out;
    -o-transition: 0.2s ease-in-out;
    -moz-transition: 0.2s ease-in-out;
    transform: rotate(0deg) scale(0);
    -ms-transform: rotate(0deg) scale(0);
    border-right: 2px solid #fff;
    border-bottom: 2px solid #fff;
  }

  .button.checkbox_false_full_focus {
    border-color: #ccc;
  }

  .button.checkbox_true_full,
  .button.checkbox_true_full_focus,
  .button.checkbox_true_part,
  .button.checkbox_true_part_focus,
  .button.checkbox_true_disable {
    border-color: #39f;
    background-color: #39f;
  }

  .button.checkbox_true_full:after,
  .button.checkbox_true_full_focus:after,
  .button.checkbox_true_disable:after {
    transform: rotate(45deg) scale(1);
    -ms-transform: rotate(45deg) scale(1);
  }

  .button.checkbox_true_part:after,
  .button.checkbox_true_part_focus:after {
    top: 5px;
    left: 2px;
    width: 8.5px;
    height: 1px;
    transform: rotate(0deg) scale(1);
    -ms-transform: rotate(0deg) scale(1);
    border-right: 0;
  }

  .button.checkbox_true_disable,
  .button.checkbox_false_disable,
  .chk.radio_false_disable,
  .chk.radio_true_disable {
    cursor: not-allowed;
  }

  .button.checkbox_false_disable {
    background-color: #f3f3f3;
  }

  .button.noline_close,
  .button.noline_open,
  .button.root_open,
  .button.root_close,
  .button.roots_open,
  .button.roots_close,
  .button.bottom_open,
  .button.bottom_close,
  .button.center_open,
  .button.center_close {
    background: none;
  }

  .button.noline_close:before,
  .button.noline_open:before,
  .button.root_open:before,
  .button.root_close:before,
  .button.roots_open:before,
  .button.roots_close:before,
  .button.bottom_open:before,
  .button.bottom_close:before,
  .button.center_open:before,
  .button.center_close:before {
    position: absolute;
    top: 5px;
    left: 5px;
    content: "";
    transition: ease 0.3s;
    -moz-transition: ease 0.3s;
    -ms-transition: ease 0.3s;
    -o-transition: ease 0.3s;
    -webkit-transition: ease 0.3s;
    transform: rotateZ(0deg);
    -ms-transform: rotateZ(0deg);
    -webkit-transform-origin: 25% 50%;
    transform-origin: 25% 50%;
    border: 6px solid;
    border-color: transparent transparent transparent #666;
  }

  .button.noline_open:before,
  .button.root_open:before,
  .button.roots_open:before,
  .button.bottom_open:before,
  .button.center_open:before {
    transform: rotateZ(90deg);
  }

  li {
    line-height: $--tree-item-height;
    list-style-type: none;
    white-space: nowrap;
    outline: none;

    ul {
      margin: 0;
      position: relative;
      padding: 0 0 0 20px;
    }

    span {
      height: $--tree-item-height;
      line-height: $--tree-item-height;
    }

    span.button.ico_close,
    span.button.ico_open,
    span.button.ico_docu {
      margin-right: 5px;
      vertical-align: middle;
    }

    a,
    a.curSelectedNode {
      height: $--tree-item-height;
      padding-top: 0;
      border: none;
      text-decoration: none;
      cursor: pointer;
    }

    a:hover {
      text-decoration: none;
    }

    .tree-node-icon {
      margin: 0 0 0 5px;
    }
  }
}

.v-theme--dark {

  .search-icon {
    color: $--search-black-border-color;
  }

  .search-input {
    border-color: $--search-black-border-color;
  }

  .tree-item {
    color: #a29b9b;
  }

  .key-tree {
    li {
      a {
        color: rgba(255, 255, 255, 0.7);
      }

      a.curSelectedNode {
        color: white;
        font-weight: bold;
      }

      a:hover {
        text-decoration: none;
        color: white;
      }
    }
  }
}

.v-theme--light {

  .search-icon {
    color: $--search-white-border-color;
  }

  .search-input {
    border-color: $--search-white-border-color;
  }

  .tree-item {
    color: black;
  }

  .key-tree {
    li {
      a {
        color: rgba(0, 0, 0, 0.7);
      }

      a.curSelectedNode {
        color: black;
        font-weight: bold;
      }

      a:hover {
        color: black;
      }
    }
  }
}

.contextmenu {
  display: none;
  position: fixed;
  z-index: 10000;
  width: 260px;
  height: fit-content;

  .v-list {
    padding: 5px 0;

    .v-list-item {
      padding-inline: 8px;
      font-size: 0.85em;
      min-height: 30px;
    }
  }
}
</style>