import {KeyExtendInfo} from "~/common/transport/kv.ts";

export interface TreeNode {
    //  节点ID，整棵树一定不能重复
    id: string,
    //  父节点ID
    pId?: string | null,
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
    //  初始化节点数据时，由 zTree 增加此属性，请勿提前赋值
    tId?: string,
    keyInfo?: KeyExtendInfo,

    getParentNode?(): TreeNode | null,
}

export function _deepSearchTreeNodes(node: TreeNode, array: TreeNode[]) {
    array.push(node)
    if (node.children) {
        for (const n of node.children) {
            _deepSearchTreeNodes(n, array)
        }
    }
}