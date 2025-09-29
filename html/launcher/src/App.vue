<script setup lang="ts">
import { RouterView } from "vue-router";
</script>

<template>
    <main class="flex flex-col items-center min-h-screen">
        <RouterView name="Search" />
        <RouterView name="Content" />
    </main>
</template>

<style scoped></style>
