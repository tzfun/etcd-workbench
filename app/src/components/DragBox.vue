<script setup lang="ts">
import {onMounted, onUnmounted, reactive, ref} from "vue";
const props = defineProps({
  itemMinWidth: {
    type: Number,
    default: 200
  },
})
const parentResizeObserver = ref<ResizeObserver>()

const dragBoxRef = ref<null | HTMLDivElement>(null)
const resizeItem = reactive({
  targetBox: <Element | null>null,
  rightBox: <Element | null> null,
  targetBoxWidth: <number>0,
  otherBoxWidth: <number>0,
  startX: <number>0,
})

onMounted(async () => {
  if (dragBoxRef.value && dragBoxRef.value.parentElement) {
    const parentElement = dragBoxRef.value.parentElement
    const resizeObserver = new ResizeObserver(entries => {
      for (let entry of entries) {
        if (entry.target == parentElement) {
          resizeLastItem()
        }
      }
    })
    resizeObserver.observe(parentElement)
    parentResizeObserver.value = resizeObserver
  }

  initDragItem()
})

onUnmounted(() => {
  if (parentResizeObserver.value && dragBoxRef.value && dragBoxRef.value.parentElement) {
    parentResizeObserver.value.unobserve(dragBoxRef.value.parentElement)
    parentResizeObserver.value = undefined
  }
})

const initDragItem = () => {
  const boxEle = dragBoxRef.value!;

  const resizeLines: HTMLCollectionOf<Element> = boxEle.getElementsByClassName("drag-resize-line")
  for (let line of resizeLines) {
    line.addEventListener('mousedown', onMouseDown)
  }
}

const resizeLastItem = () => {
  const boxEle = dragBoxRef.value!;
  if (!boxEle) {
    return
  }
  let leftWidth = 0
  let childLen = boxEle.children.length
  const parentWidth = boxEle.clientWidth
  for (let i = 0; i < childLen; i++) {
    let child = boxEle.children[i] as Element
    if(i < childLen - 1) {
      leftWidth += child.clientWidth
    } else {
      child.setAttribute("style", `width:${parentWidth - leftWidth}px`)
    }
  }
}

const onMouseDown = (e: Event) => {
  resizeItem.targetBox = (e.target as Element).parentElement!
  resizeItem.rightBox = getNextElement(resizeItem.targetBox!)
  if (!resizeItem.rightBox) {
    return
  }
  resizeItem.targetBoxWidth = resizeItem.targetBox.clientWidth
  resizeItem.otherBoxWidth = dragBoxRef.value!.clientWidth - resizeItem.targetBox.clientWidth - resizeItem.rightBox.clientWidth

  resizeItem.startX = (e as MouseEvent).clientX
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

const onMouseMove = (e: Event) => {
  const endX = (e as MouseEvent).clientX
  const moveLen = endX - resizeItem.startX
  const targetBoxWidth = resizeItem.targetBoxWidth + moveLen
  const rightBoxWidth = dragBoxRef.value!.clientWidth - targetBoxWidth - resizeItem.otherBoxWidth

  if (targetBoxWidth < props.itemMinWidth || rightBoxWidth < props.itemMinWidth) {
    return
  }

  resizeItem.targetBox!.setAttribute('style', `width:${targetBoxWidth}px;`)
  resizeItem.rightBox!.setAttribute('style', `width:${rightBoxWidth}px;`)
}

const onMouseUp = () => {
  document.removeEventListener('mousedown', onMouseDown)
  document.removeEventListener('mousemove', onMouseMove)
}

const getNextElement = (element: Element) : Element | null => {
  if (element.nextElementSibling) {
    return element.nextElementSibling
  } else {
    let next  = element.nextSibling
    while (next && next.nodeType !== 1) {
      next = next.nextSibling
    }
    return next as Element
  }
}

</script>

<template>
  <div class="drag-box" ref="dragBoxRef">
    <slot></slot>
  </div>
</template>

<style scoped lang="scss">
.drag-box {
  display: flex;
  width: 100%;
  height: 100%;
}
</style>