<template>
    <div>
        <!--NavBar-->
        <div class="absolute z-50 w-full">
            <!--Logo-->
            <img class="h-12" src="@/assets/img/landscapelogo.png" />
            <!--End Logo-->

            <nav>
                <ul>
                    <li>
                        <router-link to="/">Home</router-link>
                    </li>
                    <li>
                        <router-link to="/about">About</router-link>
                    </li>
                    <li>
                        <router-link to="/users">Users</router-link>
                    </li>
                </ul>
            </nav>
        </div>
        <div
            class="flex flex-col items-center justify-center h-screen bg-rh-blue">
            <div class="mb-3 text-6xl font-archive">
                <span class="text-rh-green">VID</span
                ><span class="text-rh-lightblue">MESTE</span>
            </div>
            <button class="px-6 py-2 rounded-full bg-rh-lightblue">
                Login
            </button>
        </div>
    </div>
</template>

<script>
export default {
    name: "Home-Page",
};
</script>

<style></style>
