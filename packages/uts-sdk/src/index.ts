import init, {
    merge_timestamps,
    pack_detached_timestamp,
    trace_timestamp
} from "uts-core-wasm";

export type HexString = string;

export type DigestOp = "SHA1" | "SHA512" | "RIPEMD160" | "KECCAK256";

export interface DigestHeader {
    kind: DigestOp;
    digest: HexString;
}


export interface BaseExecutionStep {
    input: HexString;
    output: HexString;
}

export interface DataExecutionStep extends BaseExecutionStep {
    op: "APPEND" | "PREPEND";
    data: HexString;
}

export interface UnaryExecutionStep extends BaseExecutionStep {
    op: DigestOp | "REVERSE" | "HEXLIFY";
}

export type ExecutionStep = DataExecutionStep | UnaryExecutionStep;

export type AttestationStep =
    | { kind: "pending"; url: string }
    | { kind: "bitcoin"; height: number }
    | { kind: "unknown"; tag: HexString; data: HexString };

export type TraceNode = ExecutionStep | AttestationStep | TraceNode[];

export type TraceResult = TraceNode[];

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

    packDetachedTimestamp(digest: DigestHeader, timestamp: Uint8Array): Uint8Array {
        this.checkInit();
        return pack_detached_timestamp(digest, timestamp);
    }

    traceTimestamp(timestamp: Uint8Array): TraceResult {
        this.checkInit();
        return trace_timestamp(timestamp) as TraceResult;
    }

    private checkInit() {
        if (!this.initialized) {
            throw new Error("UtsSDK not initialized. Call ensureInit() first.");
        }
    }
}
