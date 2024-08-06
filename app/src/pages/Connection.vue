<script setup lang="ts">
import {onActivated, PropType, ref} from "vue";
import {SessionData} from "~/common/transport/connection.ts";
import Cluster from "~/pages/Cluster.vue";
import Keys from "~/pages/Keys.vue";
import Users from "~/pages/Users.vue";
import Roles from "~/pages/Roles.vue";

let activeListItem = ref<string>('cluster')

const props = defineProps({
  session: {
    type: Object as PropType<SessionData>,
    required: true
  }
})

onActivated(() => {
  console.log("mounted", props.session)
})

const selectList = ({id}) => {
  console.log(id)
}

const clickList = (page: string) => {
  activeListItem.value = page
}

</script>

<template>
  <v-layout class="fill-height overflow-y-auto">
    <v-navigation-drawer permanent>
      <v-list lines="two"
              activatable
              :activated="activeListItem"
              color="primary"
              @click:activate="selectList"
      >
        <v-divider></v-divider>
        <v-list-item value="cluster"
                     prepend-icon="mdi-apps"
                     @click="clickList('cluster')"
        >Cluster
        </v-list-item>
        <v-list-item value="keys"
                     prepend-icon="mdi-database"
                     @click="clickList('keys')"
        >Keys
        </v-list-item>
        <v-list-item value="users"
                     prepend-icon="mdi-account-supervisor"
                     @click="clickList('users')"
                     :disabled="!session.root"
        >Manage Users
        </v-list-item>
        <v-list-item value="roles"
                     prepend-icon="mdi-lock"
                     @click="clickList('roles')"
                     :disabled="!session.root"
        >Manage Roles
        </v-list-item>
      </v-list>
    </v-navigation-drawer>
    <v-main class="fill-height">
      <Cluster v-show="activeListItem == 'cluster'"
               :session="session"
      ></Cluster>
      <Keys v-show="activeListItem == 'keys'"
            :session="session"></Keys>
      <Users v-show="activeListItem == 'users'"
             :session="session"></Users>
      <Roles v-show="activeListItem == 'roles'"
             :session="session"></Roles>
    </v-main>
  </v-layout>
</template>

<style scoped>

</style>