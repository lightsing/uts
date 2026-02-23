# OpenTimestamps End2End Procedures

This is for referencing overall end2end procedures of OpenTimestamps protocol implementation.

> opentimestamps clients has a white list of default servers (see below).
> Bad news is Even if we are fully compatible with ots, ots clients won't work with our servers.
> Good news is we can use ots servers in our clients.

---

## Client Side

1. Calc initial digest from (data, file) and adding nonce (random generated) to it.
   This allows to separate the sub-timestamp for individual file without leaking any information about the adjacent files.
2. Construct Merkle tree.
3. Submit tree root digest to **aggregator** servers in parallel via [POST /digest](./ots-api.md#post-digest) endpoint.
4. Merge received pending attestations into a single attestation file.
5. Periodically check for completed attestations to **calendar** servers via [GET /attestation/{digest}](./ots-api.md#get-timestamphex_commitment) endpoint.

---

## Aggregator Side

1. Receive submitted digest via [POST /digest](./ots-api.md#post-digest) endpoint.
2. Insert the digest into local Merkle tree.
3. Periodically (default interval is 1s) submit the tree root to upstream calendar servers via [POST /digest](./ots-api.md#post-digest) endpoint.
4. **Return** pending attestation for the submitted digest.

The aggregator only has `/digest` endpoint, and do not persist data.

Default ots aggregator servers:

- https://a.pool.opentimestamps.org
- https://b.pool.opentimestamps.org
- https://a.pool.eternitywall.com
- https://ots.btc.catallaxy.com

---

## Calendar Server Side

**Note: the following procedure is not a part of OpenTimestamps protocol.**
**Server does not have to follow steps 2-3 below.**

### Digest Submission Procedure

1. Receive submitted digest via [POST /digest](./ots-api.md#post-digest) endpoint.
2. Add `PREPEND` op: prepends current timestamp (u32) to the digest.
3. Add `APPEND` op: HMAC with server secret to the digest, truncated to 8 bytes, appended.
   After this step, the digest becomes the attestation message (44 bytes).
4. Add `ATTESTATION` op: uri with self.
5. Insert the step 3 **message** into journal as pending attestation. (just a file opened for append in python implementation)
6. **Return** pending attestation for the request.

### Attestation upgrade Procedure

1. Receive upgrade request via [GET /timestamp/{hex_commitment}](./ots-api.md#get-timestamphex_commitment) endpoint.
2. Search for the commitment in db.

---

Default calendar servers:

- https://\*.calendar.opentimestamps.org', # Run by Peter Todd
- https://\*.calendar.eternitywall.com', # Run by Riccardo Casatta
- https://\*.calendar.catallaxy.com', # Run by Bull Bitcoin

---

## Stamper Side

1. Stream reading pending attestations from journal file.
2. When reaching certain batch size or time interval, construct a Merkle tree from the batch of attestation messages by hashing them.
3. Submit the tree root to Bitcoin network via `OP_RETURN` transaction.
4. After certain number of confirmations, generate and write back serialized timestamp for each attestation in the batch.
5. Repeat from step 1.
