# Security Policy

## Reporting a Vulnerability

The UTS team takes security issues seriously. We appreciate your efforts to
responsibly disclose any vulnerabilities you find.

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

- **GitHub Private Vulnerability Reporting**: Use the
  [Security Advisories](https://github.com/lightsing/uts/security/advisories)
  page to privately report a vulnerability.
- **Email**: Send an email to **light.tsing@gmail.com** with the subject line
  `[UTS Security]` followed by a brief description.

### What to Include

Please include as much of the following information as possible to help us
understand and reproduce the issue:

- Description of the vulnerability
- Steps to reproduce or proof-of-concept
- Affected component(s) (e.g., `uts-core`, `uts-calendar`, `uts-relayer`,
  smart contracts)
- Impact assessment (what an attacker could achieve)
- Any suggested fix or mitigation

### Response Timeline

- **Acknowledgment**: We will acknowledge receipt of your report within
  **48 hours**.
- **Assessment**: We will provide an initial assessment within **1 week**.
- **Fix & Disclosure**: We aim to release a fix and coordinate disclosure within
  **90 days**, depending on severity and complexity.

## Supported Versions

| Component | Version | Supported |
| --- | --- | --- |
| uts-cli | 0.1.0-alpha.x | :white_check_mark: |
| uts-core | 0.1.0-alpha.x | :white_check_mark: |
| uts-calendar | 0.1.0-alpha.x | :white_check_mark: |
| uts-relayer | 0.1.0-alpha.x | :white_check_mark: |
| Smart Contracts | Latest deployed | :white_check_mark: |

## Security Considerations

UTS is a timestamping protocol that anchors data on-chain. Key areas of security
concern include:

- **Merkle tree integrity**: Correctness of the Binary Merkle Tree
  implementation in `uts-bmt`
- **Proof verification**: Ensuring timestamp proofs cannot be forged or tampered
  with in `uts-core`
- **Smart contract security**: EAS attestation and anchoring contract
  correctness
- **Server-side security**: Calendar and relayer service hardening
- **Cross-chain anchoring**: L2-to-L1 relay integrity

## Acknowledgments

We are grateful to the security researchers and community members who help keep
UTS safe. Contributors who report valid vulnerabilities will be acknowledged
here (with their permission).
