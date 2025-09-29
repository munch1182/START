<script setup lang="ts">
import { Toaster } from 'vue-sonner'
import 'vue-sonner/style.css'
import { RouterView } from 'vue-router'
</script>

<template>
    <main>
        <Toaster />
        <RouterView />
    </main>
</template>

<style scoped></style>
