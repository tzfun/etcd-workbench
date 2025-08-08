<script setup lang="ts">
import {computed, onMounted, onUnmounted, PropType, reactive, ref} from "vue";
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
import {_disconnect, _handleError, _kvSearchNextDir, _removeKeyMonitor, _setKeyMonitor} from "~/common/services.ts";
import {_isEmpty} from "~/common/utils.ts";
import {appWindow} from "@tauri-apps/api/window";
import {Handler} from "mitt";
import CompleteInput from "~/components/CompleteInput.vue";
import {useI18n} from "vue-i18n";

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})
const {t} = useI18n();
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
          isPrefix: e.isPrefix || false,
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
    _tipWarn(t('main.roles.requiredKeyTip'))
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

const searchNextNode = (value: string | null): Promise<string[]> => {
  return searchNext(value, true)
}

const searchNext = (value: string | null, includeFile: boolean): Promise<string[]> => {
  const prefix = value || ""
  return _kvSearchNextDir(props.session?.id, prefix, includeFile).catch(e => {
    _handleError({
      e,
      session: props.session
    })
    return []
  })
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
                   :text="t('main.home.connection.tabCluster')"
        >
          <template v-slot:activator="{ props }">
            <v-list-item title="t('main.home.connection.tabCluster')"
                         v-bind="props"
                         value="cluster"
                         prepend-icon="mdi-apps"
                         @click="clickList('cluster')"
            />
          </template>
        </v-tooltip>

        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   :text="t('main.home.connection.tabKeys')">
          <template v-slot:activator="{ props }">
            <v-list-item :title="t('main.home.connection.tabKeys')"
                         v-bind="props"
                         value="keys"
                         prepend-icon="mdi-database"
                         @click="clickList('keys')"
            />
          </template>
        </v-tooltip>
        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   :text="t('main.home.connection.tabMonitor')">
          <template v-slot:activator="{ props }">
            <v-list-item :title="t('main.home.connection.tabMonitor')"
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
                   :text="t('main.home.connection.tabLeases')">
          <template v-slot:activator="{ props }">
            <v-list-item :title="t('main.home.connection.tabLeases')"
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
                   :text="t('main.home.connection.tabUsers')">
          <template v-slot:activator="{ props }">
            <v-list-item :title="t('main.home.connection.tabUsers')"
                         v-bind="props"
                         value="users"
                         prepend-icon="mdi-account-supervisor"
                         @click="clickList('users')"
                         :disabled="!session.root"
            />
          </template>
        </v-tooltip>
        <v-tooltip location="end center"
                   origin="start center"
                   no-click-animation
                   :text="t('main.home.connection.tabRoles')">
          <template v-slot:activator="{ props }">
            <v-list-item :title="t('main.home.connection.tabRoles')"
                         v-bind="props"
                         value="roles"
                         prepend-icon="mdi-lock"
                         @click="clickList('roles')"
                         :disabled="!session.root"
            />
          </template>
        </v-tooltip>
      </v-list>
    </v-navigation-drawer>
    <v-main class="fill-height">
      <div v-show="activeListItem == 'cluster'" class="fill-height">
        <Cluster :session="session" v-if="visited['cluster']"/>
      </div>
      <div v-show="activeListItem == 'keys'" class="fill-height">
        <Keys :session="session" v-if="visited['keys']"/>
      </div>
      <div v-show="activeListItem == 'keyMonitor'" class="fill-height">
        <KeyMonitor :session="session"
                    v-if="visited['keyMonitor']"
                    :events="keyMonitorEventLog.logs"
                    @on-read="onReadKeyMonitorLog"
        />
      </div>
      <div v-show="activeListItem == 'leases'" class="fill-height">
        <Leases :session="session" v-if="visited['leases']"/>
      </div>
      <div v-show="activeListItem == 'users'" class="fill-height">
        <Users :session="session" v-if="visited['users']"/>
      </div>
      <div v-show="activeListItem == 'roles'" class="fill-height">
        <Roles :session="session" v-if="visited['roles']"/>
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
      <v-card :title="t('common.monitor')"
              prepend-icon="mdi-robot"
      >
        <template #prepend>
          <v-icon color="#cc8f53">mdi-robot</v-icon>
        </template>
        <template v-slot:append>
          <v-icon class="cursor-pointer" @click="keyMonitorDialog.show = false">mdi-close</v-icon>
        </template>
        <v-card-item class="overflow-visible">

          <v-layout class="mb-5 mt-5 overflow-visible">
            <span class="inline-label input-label">{{ t('common.key') }}: </span>
            <CompleteInput
                :search-func="searchNextNode"
                v-model="keyMonitorDialog.monitor.key"
                density="comfortable"
                :prefix="session.namespace"
                hide-details
                prepend-inner-icon="mdi-file-document"
                persistent-hint
                :readonly="keyMonitorDialog.edit"
                elevation="16"
                suggestion-max-height="140"
            />
          </v-layout>

          <v-layout class="mb-5" style="z-index: unset">
            <span class="inline-label radio-label" style="line-height: 40px;">Type: </span>
            <v-radio-group v-model="keyMonitorDialog.monitor.isPrefix" inline hide-details>
              <v-radio :label="t('main.home.connection.keyOnly')" :value="false"/>
              <v-radio :label="t('main.home.connection.withPrefix')" :value="true"/>
            </v-radio-group>
          </v-layout>

          <v-layout class="mb-5" style="z-index: unset">
            <span class="inline-label checkbox-label" style="line-height: 56px;">{{ t('main.home.connection.target') }}: </span>

            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorValueChange"
                :label="t('main.home.connection.eventValueChange')"
                hide-details
            />
            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorCreate"
                :label="t('main.home.connection.eventCreate')"
                class="ml-2"
                hide-details
            />
            <v-checkbox
                v-model="keyMonitorDialog.monitor.monitorRemove"
                :label="t('main.home.connection.eventRemove')"
                class="ml-2"
                hide-details
            />
          </v-layout>
          <v-alert
              density="compact"
              class="text-medium-emphasis"
              :text="t('main.home.connection.keyMonitorAlert')"
          />
        </v-card-item>
        <v-card-actions>
          <v-btn :text="t('main.home.connection.removeMonitor')"
                 v-if="keyMonitorDialog.edit"
                 variant="flat"
                 class="text-none"
                 color="red"
                 @click="keyMonitorRemove"
          />
          <v-btn :text="keyMonitorDialog.edit ? t('common.confirm') : t('main.home.connection.addMonitor')"
                 variant="flat"
                 class="text-none"
                 color="primary"
                 :disabled="!keyMonitorDialogCanConfirm"
                 @click="keyMonitorConfirm"
          />
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-layout>
</template>

<style scoped lang="scss">
.inline-label {
  width: 90px;
}
</style>