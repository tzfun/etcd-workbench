<script setup lang="ts">
import {onActivated, onMounted, PropType, reactive, ref} from "vue";
import {KeyMonitorConfig, SessionData} from "~/common/transport/connection.ts";
import Cluster from "~/pages/main/Cluster.vue";
import Keys from "~/pages/main/Keys.vue";
import Users from "~/pages/main/Users.vue";
import Roles from "~/pages/main/Roles.vue";
import Leases from "~/pages/main/Leases.vue";
import KeyMonitor from "~/pages/main/KeyMonitor.vue";
import {_listenLocal, EventName} from "~/common/events.ts";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

let activeListItem = ref<string>('cluster')
const visited = ref<Record<string, boolean>>({
  'cluster': true
})
const keyMonitorDialog = reactive({
  show: false,
  edit: false,
  monitor: <KeyMonitorConfig> {
    key: "",
    intervalSeconds: 5,
    monitorLeaseChange: true,
    monitorValueChange: true,
    monitorCreate: true,
    monitorRemove: true,
  },
})

onMounted(() => {
  _listenLocal(EventName.EDIT_KEY_MONITOR, (e) => {
    if (e.edit) {
      keyMonitorDialog.edit = true
      keyMonitorDialog.monitor = e.monitor as KeyMonitorConfig;
    } else {
      keyMonitorDialog.edit = false
      keyMonitorDialog.monitor.key = e.key as string
      keyMonitorDialog.monitor.intervalSeconds = 5
      keyMonitorDialog.monitor.monitorLeaseChange = true
      keyMonitorDialog.monitor.monitorValueChange = true
      keyMonitorDialog.monitor.monitorCreate = true
      keyMonitorDialog.monitor.monitorRemove = true
    }

    keyMonitorDialog.show = true
  })
})

onActivated(() => {
  console.log("mounted", props.session)
})

const selectList = ({id}: any) => {
  if (!visited.value[id]) {
    visited.value[id] = true
  }
}

const clickList = (page: string) => {
  activeListItem.value = page
}

const keyMonitorConfirm = () => {

}

const keyMonitorRemove = () => {

}

</script>

<template>
  <v-layout class="fill-height">
    <v-navigation-drawer permanent
                         rail
                         class="user-select-none"
    >
      <v-list lines="two"
              activatable
              :activated="activeListItem"
              color="primary"
              @click:activate="selectList"
              mandatory
              nav
      >
        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   text="Cluster">
          <template v-slot:activator="{ props }">
            <v-list-item title="Cluster"
                         v-bind="props"
                         value="cluster"
                         prepend-icon="mdi-apps"
                         @click="clickList('cluster')"
            ></v-list-item>
          </template>
        </v-tooltip>

        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   text="Keys">
          <template v-slot:activator="{ props }">
            <v-list-item title="Keys"
                         v-bind="props"
                         value="keys"
                         prepend-icon="mdi-database"
                         @click="clickList('keys')"
            ></v-list-item>
          </template>
        </v-tooltip>
        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   text="Key Monitor">
          <template v-slot:activator="{ props }">
            <v-list-item title="Key Monitor"
                         v-bind="props"
                         value="keyMonitor"
                         prepend-icon="mdi-file-eye"
                         @click="clickList('keyMonitor')"
            ></v-list-item>
          </template>
        </v-tooltip>
        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   text="Leases">
          <template v-slot:activator="{ props }">
            <v-list-item title="Leases"
                         v-bind="props"
                         value="leases"
                         prepend-icon="mdi-clock-time-nine"
                         @click="clickList('leases')"
            ></v-list-item>
          </template>
        </v-tooltip>
        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   text="Users">
          <template v-slot:activator="{ props }">
            <v-list-item title="Users"
                         v-bind="props"
                         value="users"
                         prepend-icon="mdi-account-supervisor"
                         @click="clickList('users')"
                         :disabled="!session.root"
            ></v-list-item>
          </template>
        </v-tooltip>
        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   text="Roles">
          <template v-slot:activator="{ props }">
            <v-list-item title="Roles"
                         v-bind="props"
                         value="roles"
                         prepend-icon="mdi-lock"
                         @click="clickList('roles')"
                         :disabled="!session.root"
            ></v-list-item>
          </template>
        </v-tooltip>
      </v-list>
    </v-navigation-drawer>
    <v-main class="fill-height">
      <div v-show="activeListItem == 'cluster'" class="fill-height">
        <Cluster :session="session" v-if="visited['cluster']"></Cluster>
      </div>
      <div v-show="activeListItem == 'keys'" class="fill-height">
        <Keys :session="session" v-if="visited['keys']"></Keys>
      </div>
      <div v-show="activeListItem == 'keyMonitor'" class="fill-height">
        <KeyMonitor :session="session" v-if="visited['keyMonitor']"></KeyMonitor>
      </div>
      <div v-show="activeListItem == 'leases'" class="fill-height">
        <Leases :session="session" v-if="visited['leases']"></Leases>
      </div>
      <div v-show="activeListItem == 'users'" class="fill-height">
        <Users :session="session" v-if="visited['users']"></Users>
      </div>
      <div v-show="activeListItem == 'roles'" class="fill-height">
        <Roles :session="session" v-if="visited['roles']"></Roles>
      </div>
    </v-main>

    <!--  Key Monitor编辑弹窗 -->
    <v-dialog
        v-model="keyMonitorDialog.show"
        persistent
        width="800px"
        scrollable
    >
      <v-card title="Key Monitor"
              prepend-icon="mdi-file-eye"
      >
        <v-card-item>
          <v-layout class="mb-5">
            <span class="grant-form-label">Key: </span>
            <v-text-field v-model="keyMonitorDialog.monitor.key"
                          type="text"
                          density="comfortable"
                          :prefix="session.namespace"
                          hide-details
                          prepend-inner-icon="mdi-file-document"
                          persistent-hint
            ></v-text-field>
          </v-layout>

          <v-layout class="mb-5">
            <span class="grant-form-label">Target: </span>

            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorValueChange"
                label="Value Change"
                hide-details
            ></v-checkbox>
            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorLeaseChange"
                label="Lease Change"
                class="ml-2"
                hide-details
            ></v-checkbox>
            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorCreate"
                label="Create"
                class="ml-2"
                hide-details
            ></v-checkbox>
            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorRemove"
                label="Remove"
                class="ml-2"
                hide-details
            ></v-checkbox>
          </v-layout>

          <v-layout class="mb-5">
            <span class="grant-form-label">Interval: </span>
            <v-text-field v-model="keyMonitorDialog.monitor.intervalSeconds"
                          type="number"
                          density="comfortable"
                          hide-details
                          persistent-hint
                          suffix="S"
                          max-width="200px"
            ></v-text-field>
          </v-layout>

        </v-card-item>
        <v-card-actions>
          <v-btn text="Cancel"
                 variant="text"
                 class="text-none"
                 @click="keyMonitorDialog.show = false"
          ></v-btn>
          <v-btn text="Remove Monitor"
                 v-if="keyMonitorDialog.edit"
                 variant="flat"
                 class="text-none"
                 color="danger"
                 @click="keyMonitorRemove"
          ></v-btn>
          <v-btn :text="keyMonitorDialog.edit ? 'Confirm' : 'Add Monitor'"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 @click="keyMonitorConfirm"
          ></v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-layout>
</template>

<style scoped lang="scss">
.grant-form-label {
  display: inline-block;
  width: 90px;
  line-height: 48px;
  user-select: none;
}
</style>