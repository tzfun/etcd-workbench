import {createMemoryHistory, createRouter} from "vue-router";
import Home from "~/pages/Home.vue";
import Cluster from "~/pages/Cluster.vue";
import Keys from "~/pages/Keys.vue";
import Users from "~/pages/Users.vue";
import Roles from "~/pages/Roles.vue";
import Connection from "~/pages/Connection.vue";

export default createRouter({
    history: createMemoryHistory(),
    routes: [
        {path: '/', name: 'home', component: Home},
        {
            path: '/connection/:id',
            name: 'connection',
            component: Connection,
            redirect: '/connection/:id/cluster',
            children: [
                {path: '/connection/:id/cluster', name: 'cluster', component: Cluster},
                {path: '/connection/:id/keys', name: 'keys', component: Keys},
                {path: '/connection/:id/users', name: 'users', component: Users},
                {path: '/connection/:id/roles', name: 'roles', component: Roles},
            ]
        },
    ],
})