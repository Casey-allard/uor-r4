//! Conformance report types: results, severity levels, and report aggregation.

/// Severity level of a conformance check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// The check passed.
    Pass,
    /// The check identified a warning (non-blocking).
    Warning,
    /// The check failed (blocks conformance).
    Failure,
}

/// A single conformance check result.
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Short identifier of the validator that produced this result.
    pub validator: String,
    /// Human-readable message describing the outcome.
    pub message: String,
    /// Severity of the result.
    pub severity: Severity,
    /// Optional additional detail lines.
    pub details: Vec<String>,
}

impl TestResult {
    /// Creates a passing result.
    pub fn pass(validator: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            validator: validator.into(),
            message: message.into(),
            severity: Severity::Pass,
            details: Vec::new(),
        }
    }

    /// Creates a failure result.
    pub fn fail(validator: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            validator: validator.into(),
            message: message.into(),
            severity: Severity::Failure,
            details: Vec::new(),
        }
    }

    /// Creates a failure result with additional detail lines.
    pub fn fail_with_details(
        validator: impl Into<String>,
        message: impl Into<String>,
        details: Vec<String>,
    ) -> Self {
        Self {
            validator: validator.into(),
            message: message.into(),
            severity: Severity::Failure,
            details,
        }
    }

    /// Creates a warning result.
    pub fn warn(validator: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            validator: validator.into(),
            message: message.into(),
            severity: Severity::Warning,
            details: Vec::new(),
        }
    }

    /// Returns true if this result represents a failure.
    pub fn is_failure(&self) -> bool {
        self.severity == Severity::Failure
    }
}

/// Aggregated conformance report from all validators.
#[derive(Debug)]
pub struct ConformanceReport {
    /// All individual test results across all validators.
    pub results: Vec<TestResult>,
    /// Results from ontology-derived meta-validators (Amendment 45).
    /// These are not counted against `CONFORMANCE_CHECKS`.
    pub meta_results: Vec<TestResult>,
}

impl ConformanceReport {
    /// Creates a new empty report.
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            meta_results: Vec::new(),
        }
    }

    /// Appends a result to this report.
    pub fn push(&mut self, result: TestResult) {
        self.results.push(result);
    }

    /// Appends a meta-validator result (not counted against `CONFORMANCE_CHECKS`).
    pub fn push_meta(&mut self, result: TestResult) {
        self.meta_results.push(result);
    }

    /// Extends this report with results from another report.
    pub fn extend(&mut self, other: ConformanceReport) {
        self.results.extend(other.results);
        self.meta_results.extend(other.meta_results);
    }

    /// Returns the count of failed meta-validator checks.
    pub fn meta_failure_count(&self) -> usize {
        self.meta_results.iter().filter(|r| r.is_failure()).count()
    }

    /// Returns the count of failed checks.
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_failure()).count()
    }

    /// Returns true if all checks passed (no failures).
    pub fn all_passed(&self) -> bool {
        self.failure_count() == 0
    }
}

impl Default for ConformanceReport {
    fn default() -> Self {
        Self::new()
    }
}
