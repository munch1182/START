<script setup lang="ts">
import { onMounted, ref } from "vue";
import Logo from "./components/Logo.vue";
import Navi from "./components/Navi.vue";
import WindowHeader from "./components/WindowHeader.vue";
import type { Plugin } from "@bridge/bridge";

const plugins = ref<Plugin[]>([]);
const activeId = ref<string | undefined>();

onMounted(loadPlugins);

async function loadPlugins() {
  plugins.value = [
    {
      id: "111",
      name: `Plugin 1`,
      version: "0.1",
      url: "https://picsum.photos/200/300",
    },
    {
      id: "222",
      name: `Plugin 2`,
      version: "0.1",
      url: "https://picsum.photos/200/300",
    },
    {
      id: "333",
      name: `Plugin 3`,
      version: "0.1",
      url: "https://picsum.photos/200/300",
    },
  ];
  activeId.value = plugins.value[0].id;
}

async function select(id: string) {
  activeId.value = id;
  const a = await window.bridge?.send("select", { id, name: "111" });
  console.log(a);
}
</script>

<template>
  <div class="flex h-full flex-row">
    <aside
      class="bg-navi w-navi flex h-full flex-col overflow-y-hidden border-r border-gray-200"
    >
      <header>
        <Logo />
      </header>
      <Navi :items="plugins" :activeId="activeId" @select="select" />
    </aside>
    <main class="flex h-full flex-1 flex-col">
      <header class="h-header flex">
        <WindowHeader data-decoration />
      </header>
      <article class="bg-page h-full flex-1"></article>
    </main>
  </div>
</template>

<style scoped></style>
