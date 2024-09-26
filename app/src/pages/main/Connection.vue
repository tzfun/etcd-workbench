<script setup lang="ts">
import {onActivated, PropType, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import Cluster from "~/pages/main/Cluster.vue";
import Keys from "~/pages/main/Keys.vue";
import Users from "~/pages/main/Users.vue";
import Roles from "~/pages/main/Roles.vue";
import Leases from "~/pages/main/Leases.vue";

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
  </v-layout>
</template>

<style scoped>

</style>