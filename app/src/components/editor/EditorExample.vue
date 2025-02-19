<script setup lang="ts">

import {Codemirror} from "vue-codemirror";
import {computed} from "vue";
import {getThemeByName} from "~/components/editor/themes.ts";
import {getLanguage} from "~/components/editor/languages.ts";
import {EditorHighlightLanguage} from "~/common/types.ts";

const props = defineProps({
  content: {
    type: String,
    required: true
  },
  contentLanguage: {
    type: String,
    required: true
  },
  theme: {
    type: String,
    required: true
  }
})

const extensions = computed(() => {
  const result = []

  const languageExtension = getLanguage(props.contentLanguage as EditorHighlightLanguage)
  if (languageExtension) {
    result.push(languageExtension)
  }

  let theme = getThemeByName(props.theme)
  if (theme) {
    result.push(theme)
  }
  return result
})

</script>

<template>
  <codemirror
      :model-value="content"
      style="width: 100%;height: 100%;"
      :extensions="extensions"
      disabled
  />
</template>

<style scoped lang="scss">

</style>
