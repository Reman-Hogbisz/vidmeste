<template>
    <!--write a 404 Page with current url-->
    <div
        class="flex flex-col items-center justify-center w-screen h-screen font-bold bg-rh-blue">
        <h1 class="mb-5 text-[8rem] text-rh-white">404</h1>
        <button
            v-on:click="gotoURL('/')"
            type="button"
            class="xl:text-xl text-white bg-[#4285F4] hover:bg-[#4285F4]/90 focus:ring-4 focus:outline-none focus:ring-[#4285F4]/50 font-medium rounded-full text-sm px-5 py-2.5 text-center inline-flex items-center dark:focus:ring-[#4285F4]/55 mr-2 mb-2">
            <img src="@/assets/img/icon.svg" class="w-6 h-6 mr-2 -ml-1" />

            <router-link to="/"
                ><span class="text-white">Go Home!</span></router-link
            >
        </button>
    </div>
</template>

<script>
export default {
    name: "NotFound404",
};
</script>

<style scope></style>
