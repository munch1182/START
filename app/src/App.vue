<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import Logo from "./components/Logo.vue";
import Navi from "./components/Navi.vue";
import WindowHeader from "./components/WindowHeader.vue";
import { Command, type PluginInfo } from "@bridge/bridge";
import type WujieVue from "wujie-vue3";

const plugins = ref<PluginInfo[]>([]);
const activeId = ref<string | undefined>();
const curr = ref<PluginInfo | undefined>();

watch(activeId, async (id) => {
  if (id) {
    curr.value = plugins.value.find((p) => p.id === id);
  } else {
    curr.value = undefined;
  }
});

onMounted(loadPlugins);

async function loadPlugins() {
  await scan_home();
}

async function scan_home() {
  const scaned = await Command.scan("C:\\Users\\MING\\ws\\START\\dist");
  if (scaned) {
    plugins.value = await Command.listplugins();
    if (plugins.value[0]) select(plugins.value[0].id);
  }
}

async function select(id: string) {
  activeId.value = id;
  curr.value = plugins.value.find((p) => p.id === id);
}
</script>

<template>
  <div class="flex h-full flex-row">
    <aside
      class="bg-navi w-navi flex h-full shrink-0 flex-col overflow-y-hidden border-r border-gray-200 max-[300px]:hidden"
    >
      <header>
        <Logo />
      </header>
      <Navi :items="plugins" :activeId="activeId" @select="select" />
    </aside>
    <main class="flex h-full w-full flex-1 flex-col">
      <header class="h-header flex">
        <WindowHeader data-decoration class="shrink-0" />
      </header>
      <article class="bg-page flex h-full w-full flex-1">
        <WujieVue
          class="h-full w-full"
          v-if="curr?.path"
          :name="curr?.name"
          :url="curr?.path"
          :props="{ pluginId: activeId }"
        />
      </article>
    </main>
  </div>
</template>

<style scoped></style>
