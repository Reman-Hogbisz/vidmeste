import { createWebHistory, createRouter } from "vue-router";
import { createApp } from "vue";
import "./index.css";

import axios from "axios";
import VueAxios from "vue-axios";

import App from "@/App.vue";

const routes = [
    {
        path: "/",
        name: "Home",
        component: () => import("@/pages/Home.vue"),
    },
    {
        path: "/login",
        name: "Login",
        component: () => import("@/pages/Login.vue"),
    },
    // Error Pages
    {
        path: "/:pathMatch(.*)*",
        name: "404",
        component: () => import("@/pages/errors/404.vue"),
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
});

const app = createApp(App);
app.use(router);
app.mixin({
    methods: {
        getUser: async () => {
            let user = null;
            await axios
                .get("/api/auth/me")
                .then((response) => {
                    if (response.data.status == 200) {
                        console.log(JSON.stringify(response.data));
                        console.log(JSON.stringify(response.data.data));
                        user = response.data.data;
                    } else {
                        console.error(
                            `Failed to get user: ${response.data.message}`
                        );
                    }
                })
                .catch((error) => {
                    console.error(`Failed to get user: ${error}`);
                });
            return user;
        },
    },
});
app.use(VueAxios, axios);
app.mount("#app");
