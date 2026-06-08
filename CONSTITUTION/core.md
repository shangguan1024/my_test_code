# Core Principles

## 1. Safety First
- All user input must be validated
- No sensitive info in logs
- Secrets never in code or commits

## 2. Modularity
- Modules communicate via explicit interfaces
- Single responsibility per module
- Dependencies flow downward

## 3. Testability
- All public APIs must have tests
- Test coverage >= 80% for new code
- Integration tests for critical paths

## 4. Backward Compatibility
- No breaking changes to public APIs
- Deprecation warnings before removal

## 5. Code Quality
- No dead code or unused imports
- Consistent naming conventions
- Documentation for all public APIs
