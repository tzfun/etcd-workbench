<script setup lang="ts">
import {computed, onActivated, onMounted, onUnmounted, PropType, reactive, ref} from "vue";
import {KeyMonitorConfig, SessionData} from "~/common/transport/connection.ts";
import Cluster from "~/pages/main/Cluster.vue";
import Keys from "~/pages/main/Keys.vue";
import Users from "~/pages/main/Users.vue";
import Roles from "~/pages/main/Roles.vue";
import Leases from "~/pages/main/Leases.vue";
import KeyMonitor from "~/pages/main/KeyMonitor.vue";
import {
  _emitLocal,
  _listenLocal,
  _tipWarn,
  _unListenLocal,
  EventName,
  KeyMonitorModifiedByServerEvent,
  KeyWatchEvent
} from "~/common/events.ts";
import {_disconnect, _handleError, _removeKeyMonitor, _setKeyMonitor} from "~/common/services.ts";
import {_isEmpty} from "~/common/utils.ts";
import {appWindow} from "@tauri-apps/api/window";
import {Handler} from "mitt";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

const eventUnListens = reactive<Function[]>([])

let activeListItem = ref<string>('cluster')
const visited = ref<Record<string, boolean>>({
  'cluster': true
})
const keyMonitorDialog = reactive({
  show: false,
  edit: false,
  monitor: <KeyMonitorConfig>{
    key: "",
    isPrefix: false,
    monitorValueChange: true,
    monitorCreate: true,
    monitorRemove: true,
    paused: false,
  },
  monitorBefore: <KeyMonitorConfig | undefined>undefined,
})

const keyMonitorEventLog = reactive({
  idCounter: 1,
  unreadNum: 0,
  logs: <KeyWatchEvent[]>[]
})

const keyMonitorDialogCanConfirm = computed<boolean>(() => {
  if (keyMonitorDialog.monitorBefore) {
    return keyMonitorDialog.monitorBefore.isPrefix != keyMonitorDialog.monitor.isPrefix
        || keyMonitorDialog.monitorBefore.monitorValueChange != keyMonitorDialog.monitor.monitorValueChange
        || keyMonitorDialog.monitorBefore.monitorCreate != keyMonitorDialog.monitor.monitorCreate
        || keyMonitorDialog.monitorBefore.monitorRemove != keyMonitorDialog.monitor.monitorRemove
        || keyMonitorDialog.monitorBefore.paused != keyMonitorDialog.monitor.paused
  }
  return true
})

onMounted(async () => {
  let editKeyMonitorEventHandler: Handler<any> = (e) => {
    if (e.session == props.session?.id) {
      if (e.edit) {
        keyMonitorDialog.edit = true
        keyMonitorDialog.monitor = e.monitor as KeyMonitorConfig;
        keyMonitorDialog.monitorBefore = JSON.parse(JSON.stringify(keyMonitorDialog.monitor))
      } else {
        keyMonitorDialog.edit = false
        keyMonitorDialog.monitor = {
          key: e.key ? (e.key as string) : '',
          isPrefix: false,
          monitorValueChange: true,
          monitorCreate: true,
          monitorRemove: true,
          paused: false
        }
        keyMonitorDialog.monitorBefore = undefined
      }

      keyMonitorDialog.show = true
    }
  }
  _listenLocal(EventName.EDIT_KEY_MONITOR, editKeyMonitorEventHandler)
  eventUnListens.push(() => _unListenLocal(EventName.EDIT_KEY_MONITOR, editKeyMonitorEventHandler))

  eventUnListens.push(await appWindow.listen<KeyWatchEvent>(EventName.KEY_WATCH_EVENT, e => {
    let event = e.payload
    if (props.session!.id == event.session) {
      event.id = keyMonitorEventLog.idCounter++
      event.eventKey = event.prevKv ? event.prevKv.key : event.curKv?.key
      keyMonitorEventLog.unreadNum++
      keyMonitorEventLog.logs.unshift(event)
    }
  }))

  eventUnListens.push(await appWindow.listen<KeyMonitorModifiedByServerEvent>(EventName.KEY_MONITOR_MODIFIED_BY_SERVER, e => {
    const event = e.payload
    if (props.session!.id == event.session) {
      props.session!.keyMonitorMap![event.config.key] = event.config
    }
  }))
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }

  _disconnect(props.session?.id)
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
  let config = keyMonitorDialog.monitor
  if (_isEmpty(config.key)) {
    _tipWarn("Key cannot be empty")
    return
  }

  _setKeyMonitor(props.session?.id, config).then(() => {
    props.session!.keyMonitorMap![config.key] = config
    keyMonitorDialog.show = false
    if (!keyMonitorDialog.edit) {
      _emitLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, {
        session: props.session?.id,
        key: keyMonitorDialog.monitor.key,
        type: 'create',
        config
      })
    }
  }).catch(e => {
    _handleError({
      e,
      session: props.session
    })
  })
}

const keyMonitorRemove = () => {
  let key = keyMonitorDialog.monitor.key
  _removeKeyMonitor(props.session?.id, key).then(() => {
    delete props.session!.keyMonitorMap![key]
    keyMonitorDialog.show = false
    _emitLocal(EventName.KEY_MONITOR_CONFIG_CHANGE, {
      session: props.session?.id,
      key: keyMonitorDialog.monitor.key,
      type: 'remove'
    })
  }).catch((e) => {
    _handleError({
      e,
      session: props.session
    })
  })
}

const onReadKeyMonitorLog = (num: number) => {
  if (num > 0) {
    keyMonitorEventLog.unreadNum = Math.max(0, keyMonitorEventLog.unreadNum - 1)
  } else {
    keyMonitorEventLog.unreadNum = 0
  }
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
                         prepend-icon="mdi-robot"
                         @click="clickList('keyMonitor')"
            >
              <template #prepend>
                <v-badge
                    v-if="keyMonitorEventLog.unreadNum > 0"
                    class="text-none"
                    color="green"
                    :content="keyMonitorEventLog.unreadNum"
                >
                  <v-icon>mdi-robot</v-icon>
                </v-badge>
                <v-icon v-else>mdi-robot</v-icon>
              </template>

            </v-list-item>
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
        <KeyMonitor :session="session"
                    v-if="visited['keyMonitor']"
                    :events="keyMonitorEventLog.logs"
                    @on-read="onReadKeyMonitorLog"
        ></KeyMonitor>
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
        width="80%"
        max-width="900px"
        scrollable
    >
      <v-card title="Key Monitor"
              prepend-icon="mdi-robot"
      >
        <template #prepend>
          <v-icon color="#cc8f53">mdi-robot</v-icon>
        </template>
        <template v-slot:append>
          <v-icon class="cursor-pointer" @click="keyMonitorDialog.show = false">mdi-close</v-icon>
        </template>
        <v-card-item>
          <v-alert
              density="compact"
              text="The monitor is bound to the connection, and it will stop running when the connection session is closed."
          ></v-alert>
          <v-layout class="mb-5 mt-5">
            <span class="grant-form-label">Key: </span>
            <v-text-field v-model="keyMonitorDialog.monitor.key"
                          type="text"
                          density="comfortable"
                          :prefix="session.namespace"
                          hide-details
                          prepend-inner-icon="mdi-file-document"
                          persistent-hint
                          :readonly="keyMonitorDialog.edit"
            ></v-text-field>
          </v-layout>

          <v-layout class="mb-5">
            <span class="grant-form-label">Type: </span>
            <v-radio-group v-model="keyMonitorDialog.monitor.isPrefix" inline hide-details>
              <v-radio label="Key Only" :value="false"></v-radio>
              <v-radio label="With Prefix" :value="true"></v-radio>
            </v-radio-group>
          </v-layout>

          <v-layout class="mb-5">
            <span class="grant-form-label">Target: </span>

            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorValueChange"
                label="Value Change"
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
        </v-card-item>
        <v-card-actions>
          <v-btn text="Remove Monitor"
                 v-if="keyMonitorDialog.edit"
                 variant="flat"
                 class="text-none"
                 color="red"
                 @click="keyMonitorRemove"
          ></v-btn>
          <v-btn :text="keyMonitorDialog.edit ? 'Confirm' : 'Add Monitor'"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 :disabled="!keyMonitorDialogCanConfirm"
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