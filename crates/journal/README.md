# uts-journal

**High-performance, append-only journal designed for the Universal Timestamps (UTS) protocol.**

`uts-journal` is an embedded, lock-free, ring-buffer-based Write-Ahead Log (WAL) implemented in Rust. 
It is designed to handle extremely high throughput (target: 1M+ TPS) with sub-millisecond durability guarantees, 
specifically optimized for the low-latency requirements of timestamping services.

## Architecture & Design Rationale

### Why not Kafka?

While distributed message queues are the standard for microservices decoupling, `uts-journal` was built to satisfy a 
specific set of constraints where generic solutions fall short:

| Feature                | Distributed MQ (e.g., Kafka)   | uts-journal                          |
|------------------------|--------------------------------|--------------------------------------|
| **Topology**           | External Service (Networked)   | Embedded Library (In-Process)        |
| **Durability Latency** | Network RTT + Disk IO (~2-5ms) | Disk IO only (<1ms possible)         |
| **Throughput Control** | Limited by Network Bandwidth   | Limited by PCIe/Memory Bandwidth     |
| **Consistency**        | Eventual / ISync Replicas      | Strong Local Consistency             |
| **Resource Usage**     | Heavy (JVM/Broker overhead)    | Minimal (Zero-copy, Zero-allocation) |

### The "Request-Persist-Return" Loop

UTS requires a synchronous acknowledgement model: **HTTP POST → Sequence → Persist → Return**.
To achieve 1M TPS under this constraint:

1. **Group Commit:** `uts-journal` allows thousands of concurrent write requests to queue in the ring buffer.
2. **Batched IO:** A dedicated WAL worker thread detects pending writes and flushes them to stable storage in minimal syscalls.
3. **Wake-on-Persist:** Once the persist boundary advances, the worker efficiently wakes only the relevant `Waker`s associated with the committed slots.

This architecture converts random IOPS into sequential throughput, allowing the system to handle massive concurrency on a single node.
