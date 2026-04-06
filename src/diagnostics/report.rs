use super::codes::DiagnosticCode;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
}

#[derive(Debug, Default, Clone)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<Diagnostic>,
}
