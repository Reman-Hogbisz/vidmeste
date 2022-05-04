import { createWebHashHistory, createRouter } from "vue-router";
import { createApp } from "vue";
import "./index.css";

import App from "@/App.vue";

const routes = [
    {
        path: "/",
        name: "Home",
        component: () => import("@/components/Home.vue"),
    },
    {
        path: "/login",
        name: "Login",
        component: () => import("@/components/Login.vue"),
    },
    {
        path: "/login2",
        name: "Login2",
        component: () => import("@/components/Login2.vue"),
    },
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

const app = createApp(App);
app.use(router);
app.mount("#app");
