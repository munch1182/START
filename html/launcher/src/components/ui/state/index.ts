export { default as State } from "./State.vue";

export const StateValue = {
    Loading: 0,
    Error: 1,
    Empty: 2,
    Data: 3,
} as const;

export type StateType = (typeof StateValue)[keyof typeof StateValue];
