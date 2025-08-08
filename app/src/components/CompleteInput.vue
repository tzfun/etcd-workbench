<script setup lang="ts">

import {onMounted, PropType, ref, watch} from "vue";
import {_debounce} from "~/common/utils.ts";
import {VListItem, VTextField} from "vuetify/components";
import {useI18n} from "vue-i18n";

type SearchFunc = (s: string) => Promise<string[]>;

const {t} = useI18n()
const props = defineProps({
  modelValue:{
    type: String,
    required: true
  },
  searchFunc: {
    type: Function as PropType<SearchFunc>,
    required: true
  },
  density: String as PropType<'default' | 'comfortable' | 'compact'>,
  appendIcon: String,
  appendInnerIcon: String,
  prependInnerIcon: String,
  prefix: String,
  hint: String,
  persistentHint: Boolean,
  hideDetails: [Boolean, String] as PropType<boolean | 'auto'>,
  variant: String as PropType<"filled" | "underlined" | "outlined" | "plain" | "solo" | "solo-inverted" | "solo-filled">,
  clearable: Boolean,
  singleLine: Boolean,
  readonly: Boolean,
  placeholder: String,
  elevation: [Number, String],
  suggestionMaxHeight: {
    type: [Number, String],
    default: () => 300
  }
})
const emits = defineEmits(['update:modelValue', 'click:append', 'click:appendInner'])

const inputValue = ref('')
const suggestions = ref<string[]>([])
const highlightedIndex = ref(-1)
const showSuggestions = ref(false)
const inputFieldRef = ref<VTextField>()
const suggestionItemRef = ref<VListItem[]>()
const searchLoading = ref(false)

watch(
    () => inputValue.value,
    (v) => {
      emits('update:modelValue', v);
    }
)

onMounted(() => {
  inputValue.value = props.modelValue
})

const search = _debounce((value: string) => {
  if (props.readonly) {
    return
  }
  searchLoading.value = true;
  props.searchFunc(value).then((data) => {
    suggestions.value = data;
    showSuggestions.value = true;
  }).finally(() => {
    searchLoading.value = false;
  })
}, 300)

const handleInput = () => {
  highlightedIndex.value = -1
  search(inputValue.value)
}

const onFocusout = () => {
  setTimeout(() => {
    showSuggestions.value = false;
  }, 200)
}

// 选择结果项
const selectItem = (item: string) => {
  // 追加选中的内容（带空格分隔）
  inputValue.value = (inputValue.value || '') + item;

  // 重置状态
  suggestions.value = [];
  highlightedIndex.value = -1;
  showSuggestions.value = false;

  // 触发新的搜索
  search(inputValue.value);

  // 保持输入框焦点
  inputFieldRef.value?.focus();
}

// 键盘导航 - 下箭头
const onArrowDown = (e: Event) => {
  e.preventDefault();
  if (!showSuggestions.value) return;
  if (suggestions.value.length == 0) return;

  highlightedIndex.value = (highlightedIndex.value + 1) % suggestions.value.length;
  scrollToHighlighted();
}

// 键盘导航 - 上箭头
const onArrowUp = (e: Event) => {
  e.preventDefault();
  if (!showSuggestions.value) return;

  if (highlightedIndex.value <= 0) {
    highlightedIndex.value = suggestions.value.length - 1;
  } else {
    highlightedIndex.value--;
  }
  scrollToHighlighted();
}

// 键盘确认选择
const onEnter = (e: Event) => {
  if (highlightedIndex.value >= 0 && showSuggestions.value) {
    e.preventDefault();
    selectItem(suggestions.value[highlightedIndex.value]);
  }
}

// ESC键关闭建议
const onEscape = () => {
  showSuggestions.value = false;
  highlightedIndex.value = -1;
}

// 滚动到高亮项
const scrollToHighlighted = () => {
  if (suggestionItemRef.value) {
    const target = suggestionItemRef.value[highlightedIndex.value]
    if (target) {
      target.$el.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
      })
    }
  }
  // 实际应用中可能需要实现滚动逻辑
  // 这里Vuetify列表会自动处理滚动
}
</script>

<template>
  <div class="search-container">
    <!-- 输入框 -->
    <v-text-field
        v-model="inputValue"
        ref="inputFieldRef"
        :density="props.density"
        :hint="hint"
        :variant="props.variant"
        :append-icon="props.appendIcon"
        :append-inner-icon="props.appendInnerIcon"
        :prepend-inner-icon="props.prependInnerIcon"
        :prefix="props.prefix"
        :persistent-hint="props.persistentHint"
        :loading="searchLoading"
        :hide-details="hideDetails"
        :clearable="props.clearable"
        :single-line="props.singleLine"
        :placeholder="props.placeholder"
        :readonly="props.readonly"
        @input="handleInput"
        @focusin="search(inputValue)"
        @focusout="onFocusout"
        @keydown.down="onArrowDown"
        @keydown.up="onArrowUp"
        @keydown.enter="onEnter"
        @keydown.esc="onEscape"
        @click:append="emits('click:append')"
        @click:appendInner="emits('click:appendInner')"
        autocomplete="off"
    ></v-text-field>

    <!-- 搜索结果列表 -->
    <v-card
        border
        :elevation="props.elevation"
        v-if="showSuggestions && suggestions && suggestions.length > 0"
        class="suggestion-container"
    >
      <v-list
          class="suggestion-list"
          :max-height="props.suggestionMaxHeight"
      >
        <v-list-item
            ref="suggestionItemRef"
            v-for="(item, index) in suggestions"
            :key="index"
            @click="selectItem(item)"
            :active="index === highlightedIndex"
        >
          {{ item }}
        </v-list-item>
      </v-list>
      <v-divider/>
      <v-card-actions class="text-medium-emphasis">
        <v-spacer/>
        <i class="notice">{{ t('component.completeInput.notice') }}</i>
      </v-card-actions>
    </v-card>
  </div>

</template>

<style scoped lang="scss">
.search-container {
  position: relative;
  width: 100%;

  .suggestion-container {
    position: absolute;
    width: 100%;
    z-index: 100;

    .suggestion-list {
      overflow-y: auto;
    }

    .notice {
      font-size: 0.9em;
    }
  }
}




</style>