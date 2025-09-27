<script setup lang="ts">
import { defineProps, onMounted, ref, } from 'vue'
import TableView from '@/components/TableView.vue';
import Button from '@/components/ui/button/Button.vue';
import { get, post } from '@/lib/net';

const devs = ref<Array<{ index: number, dev: string }>>();
const props = defineProps<{ id: string }>();
const isConnect = ref(false);
onMounted(() => { scan() })

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


</script>

<template>
    <div class="flex flex-col flex-1" id="123">
        <div class="flex flex-row flex-1 gap-2 p-2">
            <Button class="flex" @click="scan()">Scan</Button>
            <Button class="flex" @click="disconnect()">Disconnet</Button>
        </div>
        <div class="relative flex flex-1" v-if="devs">
            <TableView class="z-0 relative" :value=devs v-on:item-click="connect($event.index)"
                :show-action-column="true">
                <template #actionCell="{ item }">
                    <Button variant="secondary" @click="connect(item.index)">Connect</Button>
                </template>
            </TableView>
        </div>
    </div>
</template>

<style scoped></style>