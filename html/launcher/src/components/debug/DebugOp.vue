<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Table, TableBody, TableHeader, TableRow, TableCell, TableHead } from '@/components/ui/table'
import { get, post } from '@/net';
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

const components: { [key: string]: Component } = {
    DebugAdb
};

type Plugin = {
    id: string,
    name: string,
    keyword: string | undefined,
    version: string
}

const plugins = ref<Array<Plugin> | null>();
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
    <div class="flex justify-center pl-16 pr-16">
        <component :is="component" v-if="item" :id="id" />
        <Table class="border border-gray-300" v-if="!item">
            <TableHeader>
                <TableRow class="flex flex-1 pl-2 pr-2">
                    <TableHead class="item">ID</TableHead>
                    <TableHead class="item">Name</TableHead>
                    <TableHead class="item">Version</TableHead>
                    <TableHead class="item">Keyword</TableHead>
                    <TableHead class="item">Op</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                <TableRow v-if="!plugins || !plugins.length" class="flex flex-1">
                    <span class="item p-15">empty</span>
                </TableRow>
                <TableRow v-for="plugin in plugins" :key="plugin.name" class="flex flex-1 pl-2 pr-2"
                    @click="showItem(plugin)">
                    <TableCell class="item">{{ plugin.id }}</TableCell>
                    <TableCell class="item">{{ plugin.name }}</TableCell>
                    <TableCell class="item">{{ plugin.version }}</TableCell>
                    <TableCell class="item">{{ plugin.keyword }}</TableCell>
                    <TableCell class="item">
                        <Button variant="secondary" @click="del(plugin.id)">Del</Button>
                    </TableCell>
                </TableRow>
            </TableBody>
        </Table>
    </div>
</template>

<style scoped>
.item {
    @apply flex flex-1 text-center justify-center items-center text-ellipsis overflow-hidden;
}
</style>