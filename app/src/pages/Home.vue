<script setup lang="ts">

import Connector from "~/components/Connector.vue";
import {_getConnectionList, _handleError, _removeConnection} from "~/common/services.ts";
import {_alertError, _confirm} from "~/common/events.ts";
import {onActivated, onMounted, ref} from "vue";
import {ConnectionInfo, DEFAULT_CONNECTION} from "~/common/transport/connection.ts";

const connectionList = ref<ConnectionInfo[]>([])
const currentConnection = ref<ConnectionInfo>(DEFAULT_CONNECTION)

onActivated(() => {
  loadConnectionList()
})

onMounted(() => {
  loadConnectionList()
})

const loadConnectionList = () => {
  _getConnectionList().then(list => {
    list.sort((a, b) => a.name.localeCompare(b.name))
    connectionList.value = list
  }).catch(e => {
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
    }).catch(e => {
      _handleError({ e })
    })
  }).catch(() => {

  })
}
</script>

<template>
  <v-layout class="fill-height overflow-y-auto">
    <v-navigation-drawer permanent>
      <v-list lines="two"
              activatable
              activated="new"
              color="primary"
              @click:activate="selectConnection"
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
        >
          {{ item.name }}
          <template v-slot:prepend>
            <v-avatar>
              <v-icon>mdi-database-outline</v-icon>
            </v-avatar>
          </template>
          <template v-slot:append>
            <v-icon @click="removeConnectionConfig(item.name)">mdi-close</v-icon>
          </template>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>
    <v-main class="fill-height">
      <Connector v-model="currentConnection" @on-save="loadConnectionList"></Connector>
    </v-main>
  </v-layout>
</template>

<style scoped>

</style>
