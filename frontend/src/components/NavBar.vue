<template>
    <!--NavBar-->
    <div class="absolute z-50 w-full">
        <!--Logo-->

        <!--End Logo-->

        <!--Navigation-->
        <nav class="">
            <div class="max-w-6xl px-4 mx-auto">
                <div class="flex justify-between">
                    <div class="flex space-x-5">
                        <div>
                            <a href="#" class="flex items-center px-2 py-4">
                                <img
                                    class="h-12"
                                    src="@/assets/img/landscapelogo.png" />
                            </a>
                        </div>
                        <!-- Primary Navbar items -->
                        <!-- <div class="items-center hidden space-x-1 md:flex">
                            <a
                                href=""
                                class="px-2 py-4 font-semibold text-green-500 border-b-4 border-green-500"
                                >Home</a
                            >
                        </div> -->
                    </div>
                    <!-- Secondary Navbar items -->
                    <div class="items-center hidden space-x-3 md:flex">
                        <a
                            v-if="user"
                            href="/api/logout"
                            v-on:click="gotoURL('/api/logout')"
                            class="px-2 py-2 font-medium text-gray-500 transition duration-300 rounded bg-rh-lightblue/10 hover:bg-rh-lightblue hover:text-white"
                            >Logout</a
                        >
                    </div>
                    <!-- Mobile menu button -->
                    <div class="flex items-center md:hidden">
                        <button
                            class="p-1 text-gray-500 transition duration-300 rounded-full outline-none mobile-menu-button hover:bg-rh-lightblue hover:text-white">
                            <svg
                                class="w-8 h-8"
                                x-show="!showMenu"
                                fill="none"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                viewBox="0 0 24 24"
                                stroke="currentColor">
                                <path d="M4 6h16M4 12h16M4 18h16"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
            <!-- mobile menu -->
            <div class="hidden mobile-menu">
                <ul class="">
                    <li class="active">
                        <a
                            href="index.html"
                            class="block px-2 py-4 text-sm font-semibold text-white bg-green-500"
                            >Home</a
                        >
                    </li>
                </ul>
            </div>
        </nav>
        <!--End Navigation-->
    </div>
</template>

<script>
export default {
    name: "NavBar",
    props: {
        user: {
            type: Object,
            default: null,
        },
    },
    methods: {
        gotoURL: function (url) {
            const currentLocation = new URL(window.location);
            history.pushState({}, "", currentLocation);
            window.location.replace(url);
        },
    },
};
</script>

<style></style>
