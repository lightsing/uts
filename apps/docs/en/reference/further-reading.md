# Further Reading

## Official Resources

### Reference Book

The [Reference Book](https://book.timestamps.now) provides detailed technical documentation:

- [System Architecture](https://book.timestamps.now/architecture.html)
- [Core Primitives](https://book.timestamps.now/core-primitives/merkle-tree.html)
- [Calendar Pipeline](https://book.timestamps.now/calendar-pipeline/submission.html)
- [L1 Anchoring](https://book.timestamps.now/l1-anchoring/contracts.html)
- [Security](https://book.timestamps.now/security.html)

### GitHub

- [Main Repository](https://github.com/lightsing/uts)
- [Issues](https://github.com/lightsing/uts/issues)
- [Discussions](https://github.com/lightsing/uts/discussions)

## Related Projects

### OpenTimestamps

UTS builds upon [OpenTimestamps](https://opentimestamps.org/):

- [OpenTimestamps Documentation](https://opentimestamps.org/)
- [GitHub](https://github.com/opentimestamps)

### Ethereum Attestation Service

UTS uses [EAS](https://attest.org/) for on-chain attestations:

- [EAS Documentation](https://docs.attest.org/)
- [EAS SDK](https://github.com/ethereum-attestation-service/eas-sdk)
- [Schema Registry](https://easscan.org/)

### Scroll

L2 timestamping is deployed on [Scroll](https://scroll.io/):

- [Scroll Documentation](https://docs.scroll.io/)
- [Scroll Bridge](https://scroll.io/bridge)
- [Block Explorer](https://scrollscan.com/)

## Protocol Specifications

### Binary Format

The UTS proof format extends OpenTimestamps:

- [OTS Format Spec](https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/ots-format.md)
- [UTS Extensions](https://book.timestamps.now/core-primitives/ots-codec.html)

### Attestation Schema

UID: `0x5c5b8b295ff43c8e442be11d569e94a4cd5476f5e23df0f71bdd408df6b9649c`

EAS schema for attestations:

```
bytes32 contentHash
```
