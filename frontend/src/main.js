import { createWebHashHistory, createRouter } from "vue-router";
import { createApp } from "vue";
import "./index.css";

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
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

const app = createApp(App);
app.use(router);
app.mount("#app");
