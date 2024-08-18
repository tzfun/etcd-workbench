<script setup lang="ts">

import {Codemirror} from "vue-codemirror";
import {computed} from "vue";
import jsonLanguage from "./lang/json";
import xmlLanguage from "./lang/xml";
import yamlLanguage from "./lang/yaml";
import sqlLanguage from "./lang/sql";
import propertiesLanguage from "./lang/properties";
import {getThemeByName} from "~/components/editor/themes.ts";

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
  switch (props.contentLanguage) {
    case 'json':
      result.push(jsonLanguage())
      break
    case 'xml':
      result.push(xmlLanguage())
      break
    case 'yaml':
      result.push(yamlLanguage())
      break
    case 'sql':
      result.push(sqlLanguage())
      break
    case 'properties':
      result.push(propertiesLanguage())
      break
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
