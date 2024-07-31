<script setup lang="ts">

import Connector from "~/components/Connector.vue";

const onSelectConnection = ({id}:any) => {
  console.log(id)
}
</script>

<template>
  <v-layout class="fill-height overflow-y-auto">
    <v-navigation-drawer permanent>
      <v-list lines="two"
              activatable
              activated="new"
              @click:activate="onSelectConnection"
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
        <v-list-item value="session-1"
        >
          Session 1
          <template v-slot:prepend>
            <v-avatar>
              <v-icon>mdi-database-outline</v-icon>
            </v-avatar>
          </template>
          <template v-slot:append>
            <v-icon>mdi-close</v-icon>
          </template>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>
    <v-main class="fill-height">
      <Connector></Connector>
    </v-main>
  </v-layout>
</template>

<style scoped>

</style>