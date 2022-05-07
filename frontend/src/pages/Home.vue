<template>
    <div v-if="!loading">
        <NavBar :user="user" />
        <div
            class="flex flex-col items-center justify-center h-screen bg-rh-blue">
            <div class="mb-3 text-6xl font-archive">
                <span class="text-rh-green">VID</span
                ><span class="text-rh-lightblue">MESTE</span>
            </div>
            <button
                v-if="!user"
                v-on:click="gotoURL('/login')"
                class="px-6 py-2 text-2xl font-bold text-white transition duration-300 bg-gray-900 rounded-full hover:bg-black">
                Login
            </button>
        </div>
    </div>
    <div v-else>
        <p>Loading...</p>
    </div>
</template>

<script>
import NavBar from "@/components/NavBar.vue";

export default {
    name: "HomePage",
    components: {
        NavBar,
    },
    data() {
        return {
            user: null,
            loading: true,
        };
    },
    methods: {
        gotoURL: function (url) {
            const currentLocation = new URL(window.location);
            history.pushState({}, "", currentLocation);
            window.location.replace(url);
        },
    },
    mounted: async function () {
        this.user = await this.getUser();
        this.loading = false;
    },
};

// const btn = document.querySelector("button.mobile-menu-button");
// const menu = document.querySelector(".mobile-menu");

// btn.addEventListener("click", () => {
//     menu.classList.toggle("hidden");
// });
</script>

<style></style>
