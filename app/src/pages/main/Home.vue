<script setup lang="ts">

import Connector from "~/components/Connector.vue";
import {_getConnectionList, _handleError, _removeConnection} from "~/common/services.ts";
import {_alertError, _confirm, EventName} from "~/common/events.ts";
import {onActivated, onMounted, onUnmounted, reactive, ref} from "vue";
import {ConnectionInfo, DEFAULT_CONNECTION, ErrorPayload} from "~/common/transport/connection.ts";
import {listen} from "@tauri-apps/api/event";

const connectionList = ref<ConnectionInfo[]>([])
const currentConnection = ref<ConnectionInfo>(DEFAULT_CONNECTION)
const eventUnListens = reactive<Function[]>([])

onActivated(() => {
  loadConnectionList()
})

onMounted(async () => {
  loadConnectionList()
  eventUnListens.push(await listen(EventName.CONNECTION_IMPORTED, () => {
    loadConnectionList()
  }))
})

onUnmounted(() => {
  for (let eventUnListen of eventUnListens) {
    eventUnListen()
  }
})

const loadConnectionList = () => {
  _getConnectionList().then(list => {
    list.sort((a, b) => a.name.localeCompare(b.name))
    connectionList.value = list
  }).catch((e: ErrorPayload | string) => {
    _handleError({ e })
  })
}

const selectConnection = ({id}: any) => {
  if (id === 'new') {
    currentConnection.value = DEFAULT_CONNECTION
  } else {
    currentConnection.value = id
  }
}

const removeConnectionConfig = (name: string) => {
  _confirm('System', 'Are you sure you want to remove this configuration from your favorites list?').then(() => {
    _removeConnection(name).then(() => {
      let idx = -1
      for (let i = 0; i < connectionList.value.length; i++) {
        if (connectionList.value[i].name == name) {
          idx = i;
          break
        }
      }

      if (idx >= 0) {
        connectionList.value.splice(idx, 1)
        if (currentConnection.value.name == name) {
          currentConnection.value = DEFAULT_CONNECTION
        }
      }
    }).catch((e: ErrorPayload | string) => {
      _handleError({ e })
    })
  }).catch(() => {

  })
}
</script>

<template>
  <v-layout class="fill-height overflow-y-auto">
    <v-navigation-drawer permanent class="user-select-none">
      <v-list lines="two"
              activatable
              activated="new"
              color="primary"
              @click:activate="selectConnection"
              mandatory
              nav
      >
        <v-list-item value="new">
          New Connection
          <template v-slot:prepend>
            <v-avatar>
              <v-icon>mdi-transit-connection-variant</v-icon>
            </v-avatar>
          </template>
        </v-list-item>
        <v-list-subheader>Favorites List</v-list-subheader>
        <v-list-item v-for="item in connectionList"
                     :key="item.name"
                     :value="item"
                     class="config-close-item"
        >
          {{ item.name }}
          <template v-slot:prepend>
            <v-avatar>
              <v-icon>mdi-database-outline</v-icon>
            </v-avatar>
          </template>
          <template v-slot:append>
            <v-icon class="config-close-icon" @click="removeConnectionConfig(item.name)">mdi-close-circle</v-icon>
          </template>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>
    <v-main class="fill-height">
      <Connector v-model="currentConnection" @on-save="loadConnectionList"></Connector>
    </v-main>
  </v-layout>
</template>

<style scoped lang="scss">
.config-close-icon {
  padding: 15px;
  font-size: 0;
  transition: all 0.15s ease;
}
.config-close-item:hover {
  .config-close-icon {
    font-size: 1.5rem;
  }
}
</style>
