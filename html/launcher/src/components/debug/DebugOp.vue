<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { get, post } from '@/lib/net';
import { onMounted, ref, type Component } from 'vue';
import {
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbPage,
    BreadcrumbSeparator,
} from '@/components/ui/breadcrumb'
import DebugAdb from './DebugAdb.vue';
import TableView from '@/components/TableView.vue';

const components: { [key: string]: Component } = {
    DebugAdb
};

type Plugin = {
    id: string,
    name: string,
    keyword: string | undefined,
    version: string
}

const plugins = ref<Array<Plugin> | null | undefined>();
const item = ref<string | null>(null);
const component = ref<Component | null | undefined>(null);
const id = ref<string | null>(null);
onMounted(() => list())

async function list() {
    plugins.value = null;
    const list: Array<Plugin> | null | undefined = await get('/plugin/list');
    plugins.value = list;
}
async function scan() {
    plugins.value = null;
    const scan = await post('/plugin/scan');
    if (scan != null) {
        await list();
    }
}

async function del(id: string) {
    const res = await post('/plugin/del', { "id": id });
    if (res != null) {
        await list();
    }
}

function showItem(plugin: Plugin) {
    item.value = plugin.name
    id.value = plugin.id
    if (plugin.name == "adb") {
        component.value = components.DebugAdb
    }
}
</script>

<template>
    <div class="flex gap-1 p-8 justify-center">
        <Button variant="secondary" @click="scan()" v-if="!item">Scan</Button>
        <Button variant="secondary" @click="list()" v-if="!item">List</Button>
    </div>
    <Breadcrumb class="pl-16 pr-16">
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/">Home</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator v-if="item" />
            <BreadcrumbItem>
                <BreadcrumbPage>{{ item }}</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
    <div class="flex flex-1 pl-16 pr-16">
        <component :is="component" v-if="item" :id="id" />
        <TableView :value="plugins" :titles="['id', 'name', 'version', 'keyword',]" @itemClick="showItem" v-if="!item"
            :show-action-column="true" :clickableRows=true>
            <template #actionCell="{ item }">
                <Button variant="secondary" @click="del(item.id)">Del</Button>
            </template>
        </TableView>
    </div>
</template>

<style scoped></style>