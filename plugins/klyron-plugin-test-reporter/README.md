# Klyron Test Reporter Plugin

Generates an HTML test report from test results during the `onAfterTest` hook.

## Installation

```bash
klyron plugin install ./plugins/klyron-plugin-test-reporter
```

## Behavior

- Receives test results as JSON via the hook context
- Generates a styled HTML report at `.klyron/test-reports/test-report.html`
- Reports total, passed, failed, and skipped counts
- Displays per-suite breakdown with individual test status
