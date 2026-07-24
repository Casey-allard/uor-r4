@status:enforced
Feature: Runtime operation, allocation, and CPU portability certificates
  Performance certificates must carry explicit evidence links for declared-zero fields and record CPU portability across execution tiers.

  Scenario: Validate declared-zero evidence links in runtime performance certificate
    Given a new runtime performance certificate
    When audited for evidence link integrity
    Then all declared-zero fields contain non-empty evidence links and steady-state allocations are zero

  Scenario: Record CPU portability tier and scalar fallback confirmation
    Given a performance certificate with CPU portability record
    When checked for execution portability
    Then scalar fallback is confirmed and target tier matches the current architecture scalar-portable tier
