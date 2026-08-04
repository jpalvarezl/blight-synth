use std::{error::Error, fmt};

/// Definition category used by diagnostics without erasing the typed IDs in the
/// definition itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeCategory {
    Instrument,
    Effect,
}

/// Machine-readable reason an otherwise known, version-supported definition is
/// invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidDefinitionCode {
    InvalidKindId,
    InvalidPreparationContext,
    InvalidParameterPayload,
    ParameterOutOfRange,
    MissingResource,
    InvalidResource,
    DuplicateInstanceId,
}

/// Structured details for an invalid definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidDefinitionDiagnostic {
    pub code: InvalidDefinitionCode,
    /// Stable JSON field name when one field caused the failure.
    pub field: Option<String>,
    /// NRT-only human-readable detail. Formatting this diagnostic is not
    /// callback-safe.
    pub message: String,
}

impl InvalidDefinitionDiagnostic {
    #[must_use]
    pub fn new(
        code: InvalidDefinitionCode,
        field: Option<impl Into<String>>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            field: field.map(Into::into),
            message: message.into(),
        }
    }
}

/// Failure to resolve or prepare a serialized node definition.
///
/// The complete unknown kind and requested version are retained so an NRT
/// migration layer can report or preserve unsupported data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreparationError {
    UnknownKind {
        category: NodeCategory,
        kind: String,
        instance_id: u32,
    },
    UnsupportedSchemaVersion {
        category: NodeCategory,
        kind: String,
        instance_id: u32,
        requested: u32,
        supported: &'static [u32],
    },
    InvalidDefinition {
        category: NodeCategory,
        kind: String,
        instance_id: u32,
        schema_version: u32,
        diagnostic: InvalidDefinitionDiagnostic,
    },
}

impl fmt::Display for PreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKind {
                category,
                kind,
                instance_id,
            } => write!(
                formatter,
                "unknown {category:?} kind `{kind}` for instance {instance_id}"
            ),
            Self::UnsupportedSchemaVersion {
                category,
                kind,
                instance_id,
                requested,
                supported,
            } => write!(
                formatter,
                "unsupported {category:?} schema version {requested} for `{kind}` instance {instance_id}; supported: {supported:?}"
            ),
            Self::InvalidDefinition {
                category,
                kind,
                instance_id,
                schema_version,
                diagnostic,
            } => write!(
                formatter,
                "invalid {category:?} definition `{kind}` v{schema_version} instance {instance_id}: {:?}: {}",
                diagnostic.code, diagnostic.message
            ),
        }
    }
}

impl Error for PreparationError {}
