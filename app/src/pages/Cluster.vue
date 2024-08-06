<script setup lang="ts">

import {PropType, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import {_getCluster} from "~/common/services.ts";
import {_tipError} from "~/common/events.ts";
import {Cluster} from "~/common/transport/maintenance.ts";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})
const cluster = ref<Cluster>()

const loadCluster = () => {
  _getCluster(props.session?.id).then(c => {
    cluster.value = c
    console.log(c)
  }).catch(e => {
    _tipError(e)
  })
}
</script>

<template>
  <v-container class="overflow-y-auto">
    <v-row>
      <v-col>
        <v-btn class="text-none"
               prepend-icon="mdi-refresh"
               @click="loadCluster"
        >Refresh
        </v-btn>
      </v-col>
    </v-row>
    <div v-if="cluster">
      <v-row>
        <v-col>
          Cluster ID
          {{cluster.id}}
        </v-col>
      </v-row>
    </div>

  </v-container>
</template>

<style scoped>

</style>