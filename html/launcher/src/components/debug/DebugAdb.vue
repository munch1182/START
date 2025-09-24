<script setup lang="ts">
import { defineProps, onMounted, ref, } from 'vue'
import TableView from '@/components/TableView.vue';
import Button from '@/components/ui/button/Button.vue';
import { get, post } from '@/net';

const titles = ["index", "dev"];
const devs = ref<Array<{ index: number, dev: string }>>();
const props = defineProps<{ id: string }>();
const isConnect = ref(false);

async function scan() {
    const id = props.id
    if (!id) return
    const resp: { "devs": Array<string> } | null = await get(`/plugin/${id}/scan`);
    if (resp) {
        devs.value = resp.devs.map((dev, index) => { return { index: index, dev: dev } })
    }
}

async function disconnect() {
    const id = props.id
    if (!id) return
    const resp: boolean | null = await get(`/plugin/${id}/disconnect`);
    if (resp) {
        scan()
    }
}

async function connect(index: number) {
    const id = props.id
    if (!id) return
    isConnect.value = true
    const resp: boolean | null = await post(`/plugin/${id}/connect`, { i: index });
    if (resp) {
        scan()
    }
    isConnect.value = false
}

onMounted(() => { scan() })

</script>

<template>
    <div class="flex flex-1 flex-col">
        <div class="flex flex-row gap-2 p-2">
            <Button variant="secondary" class="flex" @click="scan">Scan</Button>
            <Button variant="secondary" class="flex" @click="disconnect">Disconnet</Button>
        </div>
        <div class="relative flex-1" v-if="devs">
            <div v-if="isConnect" class="absolute inset-0 bg-white bg-opacity-75 flex items-center justify-center z-10">
                <span>loading...</span>
            </div>
            <TableView class="z-0 relative" :titles=titles :data=devs v-on:item-click="connect($event.index)"
                :show-action-column="true">
                <template #actionCell="{ item }">
                    <Button variant="secondary" @click="connect(item.index)">Connect</Button>
                </template>
            </TableView>
        </div>
    </div>
</template>

<style scoped></style>