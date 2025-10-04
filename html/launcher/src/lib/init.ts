import { setupDragRegion, setupKeyboardShortcuts } from "./drag";

export function setup() {
    setupDragRegion(window.MSG.DRAG);
    setupKeyboardShortcuts();
}
