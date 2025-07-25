<script setup lang="ts">
import {onMounted, PropType, ref, watch} from "vue";
import {FileForm} from "~/common/types.ts";
import {_byteTextFormat, _timeFormat} from "~/common/utils.ts";
import {_dialogContent, _tipError} from "~/common/events.ts";
import {useI18n} from "vue-i18n";

const { t } = useI18n()

const props = defineProps({
  modelValue: {
    type: Object as PropType<FileForm>,
    required: true
  },
  text: String,
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

const fileInputRef = ref<HTMLInputElement>()
const modelValueMirror = ref<FileForm>(props.modelValue)

onMounted(() => {
  if (props.modelValue?.content) {
    fileReadStatus.value = 'success'
  } else {
    fileReadStatus.value = 'none'
  }
})

watch( () => props.modelValue, (newVal) => {
  modelValueMirror.value = newVal
  if (newVal?.content) {
    fileReadStatus.value = 'success'
  } else {
    fileReadStatus.value = 'none'
  }
})

const clickFileInput = () => {
  if (fileInputRef.value) {
    fileInputRef.value.click()
  }
}

const fileInputChange = (event: Event) => {
  let ele = event.target as HTMLInputElement
  if (ele.files && ele.files.length > 0) {
    let file = ele.files[0]
    if (file.size >= props.maxSize) {
      _tipError(t('component.fileSelector.sizeTip'))
      return
    }
    modelValueMirror.value.file = file
    let reader = new FileReader();
    reader.readAsText(modelValueMirror.value.file, 'utf-8')
    // reader.readAsArrayBuffer(modelValueMirror.file)
    fileReadStatus.value = 'reading'
    reader.onload = function () {
      modelValueMirror.value.content = reader.result as string
      fileReadStatus.value = 'success'
    }
    reader.onerror = function () {
      modelValueMirror.value.file = undefined
      _tipError(`${t('component.fileSelector.readFileError')}: ${reader.error?.message}`)
      fileReadStatus.value = 'error'
    }
  } else {
    modelValueMirror.value.file = undefined
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
    >{{ text || t("common.selectFile") }}
    </v-btn>
    <p v-if="promptText" class="v-messages mt-2">{{ promptText }}</p>
    <div class="file-detail mt-2">
      <div class="d-flex" v-if="fileReadStatus != 'none'">
        <span class="label text-cyan">{{ t('common.file') }}:</span>
        <span class="file-name mr-2" v-if="modelValue.file">{{ modelValue.file.name }}</span>

        <v-progress-circular
            v-if="fileReadStatus == 'reading'"
            color="grey-lighten-1"
            size="16"
            width="2"
            indeterminate
        />
        <v-btn v-else-if="fileReadStatus == 'success'"
               class="text-none"
               density="comfortable"
               size="small"
               title="View Content"
               color="green"
               variant="text"
               @click="showFileContent"
        >{{ t('common.view') }}</v-btn>
        <span v-else-if="fileReadStatus == 'error'"
              class="text-red"
        >{{ t('common.readError') }}</span>
      </div>

      <div v-if="modelValue.file">
        <span class="label text-cyan">{{ t('common.size') }}:</span>
        <span>{{ _byteTextFormat(modelValue.file.size) }}</span>
      </div>

      <div v-if="modelValue.file">
        <span class="label text-cyan">{{ t('common.lastModified') }}:</span>
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
