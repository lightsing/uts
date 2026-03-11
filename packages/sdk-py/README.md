# UTS Python SDK

Python SDK for Universal Timestamps (UTS) - a decentralized timestamping protocol with EAS integration.

## Installation

```bash
pip install uts-python-sdk
```

## Usage

```python
from uts_sdk import UTS

async with UTS() as client:
    stamp = await client.stamp(b"Hello, World!")
    result = await client.verify(stamp)
```
