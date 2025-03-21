# Unit Testing Guide
- Purpose: Verify individual code units work correctly, detect bugs early
- Structure: Arrange (setup), Act (execute), Assert (verify)
- Qualities: Small, isolated, deterministic, independent, automated

## Write tests that:
- Test single functionality/behavior
- Have clear, descriptive names
- Mock external dependencies
- Run quickly, repeatedly
- Cover happy path + edge cases
- Fail with helpful messages
- Remain independent of other tests

## Avoid:
- Testing multiple behaviors at once
- External dependencies (DB, API)
- Non-deterministic results
- Tight coupling to implementation
- Testing trivial code
