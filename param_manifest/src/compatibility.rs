//! Compatibility comparison between two manifest versions.
//!
//! The stability rules from ADR 0003 are enforced here so a CI or review step can
//! diff a proposed manifest against the accepted one:
//!
//! * A stable ID must not disappear silently — removing a live descriptor without
//!   first deprecating it is a breaking change.
//! * A stable ID must not change its automation-rate traffic class; hosts bind
//!   overload behavior to it.
//! * Adding a new descriptor is always compatible.

use crate::descriptor::ParameterId;
use crate::manifest::ParameterManifest;

/// A single detected incompatibility between an old and new manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityBreak {
    /// A previously-published ID is gone and was not deprecated first.
    RemovedWithoutDeprecation(ParameterId),
    /// A previously-published ID changed its automation traffic class.
    AutomationRateChanged(ParameterId),
    /// A previously-published ID changed a meaning-bearing field (mapping, range,
    /// unit, kind, or engine parameter slot), which would reinterpret saved
    /// normalized automation/project values. Renames must use a new ID instead.
    SemanticsChanged(ParameterId),
}

impl std::fmt::Display for CompatibilityBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompatibilityBreak::RemovedWithoutDeprecation(id) => {
                write!(f, "parameter `{id}` was removed without prior deprecation")
            }
            CompatibilityBreak::AutomationRateChanged(id) => {
                write!(f, "parameter `{id}` changed its automation rate")
            }
            CompatibilityBreak::SemanticsChanged(id) => {
                write!(f, "parameter `{id}` changed a meaning-bearing field under the same id")
            }
        }
    }
}

/// The result of comparing a proposed manifest against a prior one.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompatibilityReport {
    /// Breaking changes; a non-empty list means the change requires an ADR/bump.
    pub breaks: Vec<CompatibilityBreak>,
    /// IDs added by the new manifest (informational, always compatible).
    pub added: Vec<ParameterId>,
    /// IDs newly marked deprecated (informational, compatible).
    pub newly_deprecated: Vec<ParameterId>,
}

impl CompatibilityReport {
    /// Whether the new manifest is backward-compatible with the old one.
    #[must_use]
    pub fn is_compatible(&self) -> bool {
        self.breaks.is_empty()
    }
}

impl ParameterManifest {
    /// Compare this manifest (the *new* one) against a `previous` manifest and
    /// report added/deprecated IDs plus any breaking changes.
    #[must_use]
    pub fn compatibility_against(&self, previous: &ParameterManifest) -> CompatibilityReport {
        let mut report = CompatibilityReport::default();

        for old in &previous.parameters {
            match self.descriptor(&old.id) {
                None => {
                    // The old ID is gone. Only allowed if it was already
                    // deprecated in the previous manifest.
                    if old.deprecated.is_none() {
                        report
                            .breaks
                            .push(CompatibilityBreak::RemovedWithoutDeprecation(old.id.clone()));
                    }
                }
                Some(new) => {
                    if new.automation_rate != old.automation_rate {
                        report
                            .breaks
                            .push(CompatibilityBreak::AutomationRateChanged(old.id.clone()));
                    }
                    // Meaning-bearing fields must not change under a stable ID;
                    // saved normalized values would be reinterpreted.
                    if new.mapping != old.mapping
                        || new.range != old.range
                        || new.unit != old.unit
                        || new.kind != old.kind
                        || new.owner.engine_param_index != old.owner.engine_param_index
                    {
                        report
                            .breaks
                            .push(CompatibilityBreak::SemanticsChanged(old.id.clone()));
                    }
                    if new.deprecated.is_some() && old.deprecated.is_none() {
                        report.newly_deprecated.push(old.id.clone());
                    }
                }
            }
        }

        for new in &self.parameters {
            if previous.descriptor(&new.id).is_none() {
                report.added.push(new.id.clone());
            }
        }

        report
    }
}
