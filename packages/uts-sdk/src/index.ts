import init, { merge_timestamps } from "uts-core-wasm";

export class UtsSDK {
    private initialized = false;

    async ensureInit() {
        if (!this.initialized) {
            await init();
            this.initialized = true;
        }
    }

    mergeTimestamps(timestamps: Uint8Array[]): Uint8Array {
        return merge_timestamps(timestamps)
    }
}
