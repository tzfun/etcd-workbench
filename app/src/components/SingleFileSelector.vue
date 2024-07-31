<script setup lang="ts">
import {PropType, ref} from "vue";
import {FileForm} from "~/common/types.ts";
import {_byteTextFormat, _timeFormat} from "~/common/utils.ts";
import {_dialogContent, _tipError} from "~/common/events.ts";

const props = defineProps({
  modelValue: {
    type: Object as PropType<FileForm>,
    required: true
  },
  text: {
    type: String,
    default: () => "Select File"
  },
  promptText: String,
  accept: {
    type: String,
    default: () => "*/*"
  },
  maxSize: {
    type: Number,
    default: 1024 * 1024
  }
})

const fileReadStatus = ref<'none' | 'reading' | 'success' | 'error'>('none')

const fileInputRef = ref(null)
const modelValueMirror = props.modelValue

const clickFileInput = () => {
  fileInputRef.value?.click()
}

const fileInputChange = (event: Event) => {
  let ele = event.target as HTMLInputElement
  if (ele.files && ele.files.length > 0) {
    let file = ele.files[0]
    if (file.size >= props.maxSize) {
      _tipError("Selected file is too large!")
      return
    }
    modelValueMirror.file = file
    let reader = new FileReader();
    reader.readAsText(modelValueMirror.file, 'utf-8');
    fileReadStatus.value = 'reading'
    reader.onload = function () {
      modelValueMirror.content = reader.result as string
      fileReadStatus.value = 'success'
    }
    reader.onerror = function () {
      modelValueMirror.file = undefined
      _tipError(`Read file error: ${reader.error?.message}`)
      fileReadStatus.value = 'error'
    }
  } else {
    modelValueMirror.file = undefined
    fileReadStatus.value = 'none'
  }
}

const showFileContent = () => {
  if (fileReadStatus.value == 'success' && props.modelValue.content) {
    _dialogContent(props.modelValue.content)
  }
}

</script>

<template>
  <div class="mt-2 mb-5">
    <v-btn
        class="text-none select-button"
        variant="outlined"
        density="comfortable"
        prepend-icon="mdi-file-document-outline"
        @click="clickFileInput"
    >{{ text }}
    </v-btn>
    <p v-if="promptText" class="prompt-text text-grey-lighten-1 mt-2">{{ promptText }}</p>
    <div class="file-detail mt-2">
      <div class="d-flex" v-if="fileReadStatus != 'none'">
        <span class="label text-cyan">File:</span>
        <span class="file-name mr-2" v-if="modelValue.file">{{ modelValue.file.name }}</span>

        <v-progress-circular
            v-if="fileReadStatus == 'reading'"
            color="grey-lighten-1"
            size="16"
            width="2"
            indeterminate
        ></v-progress-circular>
        <v-btn v-else-if="fileReadStatus == 'success'"
               class="text-none"
               density="comfortable"
               size="small"
               title="View Content"
               color="green"
               variant="text"
               @click="showFileContent"
        >view
        </v-btn>
        <span v-else-if="fileReadStatus == 'error'"
              class="text-red"
        >Read Error</span>
      </div>

      <div v-if="modelValue.file">
        <span class="label text-cyan">Size:</span>
        <span>{{ _byteTextFormat(modelValue.file.size) }}</span>
      </div>

      <div v-if="modelValue.file">
        <span class="label text-cyan">Last Modify:</span>
        <span>{{ _timeFormat(modelValue.file.lastModified) }}</span>
      </div>
    </div>

    <input type="file"
           class="d-none"
           ref="fileInputRef"
           :accept="accept"
           @change="fileInputChange"/>
  </div>
</template>

<style scoped lang="scss">

.file-detail {
  font-size: 1em;

  .file-name {
    max-width: 150px;
    text-overflow: ellipsis;
    white-space: nowrap;
    overflow: hidden;
  }

  .label {
    width: 100px;
    display: inline-block;
  }
}

</style>