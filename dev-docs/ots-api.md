# OpenTimestamps API

The API is summarized here for reference.

- Core Endpoints
 - Submit [POST /digest](#post-digest)
 - Upgrade [GET /timestamp/{hex_commitment}](#get-timestamphex_commitment)
- Other Endpoints
  - Fetch Tip [GET /tip](#get-tip)

---
## POST /digest
Submit a commitment to the calendar aggregator for stamping.

Implementation: https://github.com/opentimestamps/opentimestamps-server/blob/6309db6b2c9ac79f6f444d85c9dc96e39219eb63/otsserver/rpc.py#L48-L81

### Request Body
Raw digest bytes (≤64 bytes); Content-Length header required.

### On Success
200 application/octet-stream with serialized [Timestamp](../crates/core/src/codec/v1/timestamp.rs) tree.

### On Failure
400 invalid or missing Content-Length; 400 digest too long.

### Example

```bash
curl -X POST https://a.pool.opentimestamps.org/digest \
  -H "Content-Type: application/octet-stream" \
  --data-binary @digest.bin \
  --output tree.bin
```

---
## GET /timestamp/{hex_commitment}
Retrieve upgraded timestamp data for a commitment. Commitment must be lowercase/uppercase hex.

Implementation: https://github.com/opentimestamps/opentimestamps-server/blob/6309db6b2c9ac79f6f444d85c9dc96e39219eb63/otsserver/rpc.py#L122-L186

### On Success
200 application/octet-stream with serialized [Timestamp](../crates/core/src/codec/v1/timestamp.rs) tree;
Cache-Control max-age=31536000 once confirmed.

### On Failure
400 non-hex input; 404 with text body (Pending… or Not found) and Cache-Control max-age=60.

### Example

```bash
curl https://alice.btc.calendar.opentimestamps.org/timestamp/6938f93117b90a25a186021b8694f7d3622299aba932130adb748f1a32f21347eddfa3df99c41b4b9cb34ff8 \
  -H "Accept: application/octet-stream" \
  --output upgraded.bin
```

---
## Get /tip
Fetch the most recent unconfirmed Merkle tree tip being prepared for anchoring.

Implementation: https://github.com/opentimestamps/opentimestamps-server/blob/6309db6b2c9ac79f6f444d85c9dc96e39219eb63/otsserver/rpc.py#L83-L101

### On Success
200 application/octet-stream containing tip commitment; Cache-Control max-age=10.

### On Failure
204 when tip exists but has no payload; 404 when no unconfirmed transactions.

### Example

```bash
$ curl https://alice.btc.calendar.opentimestamps.org/tip --output tip.bin 
$ hexdump -C tip.bin
00000000  45 3f 4d 34 fd 72 c5 70  44 60 eb c4 b6 f5 09 87  |E?M4.r.pD`......|
00000010  72 7d da 8c a8 9e 00 1e  cf c7 29 17 b4 c4 1f b0  |r}........).....|
00000020
```
