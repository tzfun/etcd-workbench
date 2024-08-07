<script setup lang="ts">

import {_getAllKeys} from "~/common/services.ts";
import {_tipError} from "~/common/events.ts";
import {PropType} from "vue";
import {SessionData} from "~/common/transport/connection.ts";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const loadAllKeys = () => {
  _getAllKeys(props.session?.id).then(data => {
    console.log(data)
  }).catch(e => {
    _tipError(e)
  })
}

const addKey = () => {

}

const deleteKey = () => {

}

</script>

<template>
  <div class="fill-height pa-5 overflow-y-auto">
    <div>
      <v-btn class="text-none"
             prepend-icon="mdi-refresh"
             color="primary"
             @click="loadAllKeys"
      >Refresh</v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-file-document-plus-outline"
             color="green"
             @click="addKey"
      >
        Add Key
      </v-btn>
      <v-btn class="text-none ml-2"
             prepend-icon="mdi-file-document-minus-outline"
             color="red"
             @click="addKey"
      >
        Delete Key
      </v-btn>
    </div>
  </div>
</template>

<style scoped>

</style>