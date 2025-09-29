import { createWebHistory, createRouter } from "vue-router";
import Search from "@/components/Search.vue";
import Content from "@/components/Content.vue";

export default createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            components: {
                Search,
                Content,
            },
        },

        // {
        //     path: "/debug",
        //     component: () => import("@/components/debug/DebugOp.vue"),
        //     children: [
        //         {
        //             path: "adb",
        //             component: () => import("@/components/debug/DebugAdb.vue")
        //         }
        //     ]
        // }
    ],
});
