<script setup lang="ts">
import {onActivated, PropType, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import Cluster from "~/pages/Cluster.vue";
import Keys from "~/pages/Keys.vue";
import Users from "~/pages/Users.vue";
import Roles from "~/pages/Roles.vue";

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
  console.log(id)
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
                         expand-on-hover
    >
      <v-list lines="two"
              activatable
              :activated="activeListItem"
              color="primary"
              @click:activate="selectList"
      >
        <v-list-item title="Cluster"
                     value="cluster"
                     prepend-icon="mdi-apps"
                     @click="clickList('cluster')"
        ></v-list-item>
        <v-list-item title="Keys"
                     value="keys"
                     prepend-icon="mdi-database"
                     @click="clickList('keys')"
        ></v-list-item>
        <v-list-item title="Manage Users"
                     value="users"
                     prepend-icon="mdi-account-supervisor"
                     @click="clickList('users')"
                     :disabled="!session.root"
        ></v-list-item>
        <v-list-item title="Manage Roles"
                     value="roles"
                     prepend-icon="mdi-lock"
                     @click="clickList('roles')"
                     :disabled="!session.root"
        ></v-list-item>
      </v-list>
    </v-navigation-drawer>
    <v-main class="fill-height">
      <div v-show="activeListItem == 'cluster'" class="fill-height">
        <Cluster :session="session" v-if="visited['cluster']"></Cluster>
      </div>
      <div v-show="activeListItem == 'keys'" class="fill-height">
        <Keys :session="session" v-if="visited['keys']"></Keys>
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