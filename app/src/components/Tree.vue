<script setup lang="ts">
import {onMounted, ref} from "vue";
import {KeyValue} from "~/common/transport/kv.ts";

import '@ztree/ztree_v3/js/jquery-1.4.4.min';
import '@ztree/ztree_v3/js/jquery.ztree.core.js';
import '@ztree/ztree_v3/js/jquery.ztree.excheck.js';
// import '@ztree/ztree_v3/css/metroStyle/metroStyle.css';
import {b} from "vite/dist/node/types.d-aGj9QkWt";

export type TreeNode = {
  //  节点ID，整棵树一定不能重复
  id: string | number,
  //  父节点ID
  pId?: string | number | null,
  //  节点显示名称
  name: string,
  //  是否是父节点
  isParent: boolean,
  //  是否展开
  open: boolean,
  //  节点默认图标路径
  icon?: string,
  //  如果是父节点，关闭状态的图标路径
  iconClose?: string,
  //  如果是父节点，打开状态的图标路径
  iconOpen?: string,
  //  子节点数组
  children?: TreeNode[],
  //  鼠标hover后的title显示
  title?: string,
}

const emits = defineEmits(['on-click'])
const props = defineProps({
  treeId: {
    type: String,
    required: true
  },
  keySplitter: {
    type: String,
    default: "/"
  }
})

const treeRootObj = ref()
const treeLastSelectedItem = ref<string>()

const onClick = (e: MouseEvent, treeId: any, treeNode: TreeNode) => {
  if (treeNode.isParent) {
    treeRootObj.value.expandNode(treeNode)
    if (treeLastSelectedItem.value) {
      let node = getTreeNodeById(treeLastSelectedItem.value)
      if (node) {
        treeRootObj.value.selectNode(node)
      }
    }
  } else {
    treeLastSelectedItem.value = treeNode.id
    emits('on-click', treeNode.id)
  }
}

const showTitle = (id: string, node: TreeNode) => {
  return !node.isParent
}

const settings = {
  data: {
    key: {
      title: "title"
    }
  },
  view: {
    nodeClasses: {add: ['tree-item']},
    showLine: false,
    showTitle: showTitle,
    dblClickExpand: false,
    selectedMulti: false,
  },
  callback: {
    onClick: onClick
  },
  check: {
    enable: true
  }
}

onMounted(() => {
  rerender()
})

/**
 * 重新渲染树结构，并初始化数据
 */
const rerender = () => {
  let tree = treeRootObj.value
  if (tree) {
    $.fn.zTree.destroy(props.treeId);
  }
  treeRootObj.value = $.fn.zTree.init($(`#${props.treeId}`), settings, [])
}

/**
 * 根据ID获取树中的节点信息
 * @param id 树的id
 * @return {TreeNode | undefined}
 */
const getTreeNodeById = (id: any): TreeNode | undefined => {
  return treeRootObj.value.getNodesByParam("id", id, null)[0]
}

const addItemToTree = (key: string) => {
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
  let newNode: TreeNode = constructFileNode(id, fileName, parentNode ? parentNode.id : undefined)
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

const constructDirNode = (id: string, name: string, pId?: string): TreeNode => {
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

const constructFileNode = (id: string, name: string, pId?: string): TreeNode => {
  return {
    id: id,
    pId: pId,
    name: name,
    isParent: false,
    open: false,
    title: id,
    icon: "/file-text.png",
  }
}

defineExpose({
  addItemToTree,
  removeItemFromTree,
  rerender
})

</script>

<template>
  <div :id="treeId" class="ztree key-tree overflow-auto"></div>
</template>

<style lang="scss">
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

    a {
      opacity: 0.8;
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
      opacity: 0.9;
    }
  }
}

.v-theme--dark {
  .tree-item {
    color: #a29b9b;
  }

  .key-tree {
    li {
      a.curSelectedNode {
        color: white;
        opacity: 1;
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
  .tree-item {
    color: black;
  }

  .key-tree {
    li {
      a.curSelectedNode {
        color: black;
        opacity: 1;
        font-weight: bold;
      }

      a:hover {
        color: black;
      }
    }
  }
}

</style>