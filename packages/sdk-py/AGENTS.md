# Python SDK Development Commands

## Code Quality

- **Type check**: `cd packages/sdk-py && poetry run mypy src/`
- **Lint**: `cd packages/sdk-py && poetry run ruff check src/ tests/`
- **Format**: `cd packages/sdk-py && poetry run ruff format src/ tests/`

## Testing

- **Run all tests**: `cd packages/sdk-py && poetry run pytest`
- **Run tests with coverage**: `cd packages/sdk-py && poetry run pytest --cov=uts_sdk --cov-report=term-missing`
- **Run specific test**: `cd packages/sdk-py && poetry run pytest tests/test_sdk.py -v`

## Build

- **Build package**: `cd packages/sdk-py && poetry build`
