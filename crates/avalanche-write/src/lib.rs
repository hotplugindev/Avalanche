use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use avalanche_model::source::{Position, SourceSpan};
use avalanche_model::validation::{ValidationReport, ValidationTier};
use avalanche_model::{
    EditKind, FileChange, Fingerprint, GenerationPlan, MutationIntent, MutationPlan, NewFile,
    RenderHint, ResolvedEdit, Transaction, TransactionStatus,
};
use avalanche_source::{
    apply_patch, generate_binding_insertion_patch, generate_binding_removal_patch,
    generate_value_replacement_patch, Binding, Expr, PatchError,
};
use thiserror::Error;

pub mod templates;

pub use templates::{
    capability_module, category_default, init_skeleton, profile_module, program_module,
    CapabilitySpec, HostSpec, InitSpec, ProfileSpec, ProgramSpec, UserSpec,
};

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("ownership check failed: {0}")]
    OwnershipDenied(String),
    #[error("stale index: repository changed after indexing; nothing was modified")]
    StaleIndex,
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("edit failed: {0}")]
    EditFailed(String),
    #[error("io error while reading {path}: {reason}")]
    Io { path: String, reason: String },
    #[error("unsupported value for {option_path}: {reason}; refusing to silently transform")]
    UnsupportedValue { option_path: String, reason: String },
    #[error("unsupported operation: {0}; repository was not modified")]
    Unsupported(String),
}

pub trait FileSource {
    fn read(&self, rel_path: &str) -> Result<String, WriteError>;
    fn write(&mut self, rel_path: &str, content: &str) -> Result<(), WriteError>;
    fn remove(&mut self, rel_path: &str) -> Result<(), WriteError>;
}

pub struct DiskFileSource {
    root: PathBuf,
}

impl DiskFileSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl FileSource for DiskFileSource {
    fn read(&self, rel_path: &str) -> Result<String, WriteError> {
        let full = self.root.join(rel_path);
        std::fs::read_to_string(&full).map_err(|e| WriteError::Io {
            path: full.display().to_string(),
            reason: e.to_string(),
        })
    }

    fn write(&mut self, rel_path: &str, content: &str) -> Result<(), WriteError> {
        let full = self.root.join(rel_path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| WriteError::Io {
                path: parent.display().to_string(),
                reason: e.to_string(),
            })?;
        }
        std::fs::write(&full, content).map_err(|e| WriteError::Io {
            path: full.display().to_string(),
            reason: e.to_string(),
        })
    }

    fn remove(&mut self, rel_path: &str) -> Result<(), WriteError> {
        let full = self.root.join(rel_path);
        std::fs::remove_file(&full).map_err(|e| WriteError::Io {
            path: full.display().to_string(),
            reason: e.to_string(),
        })
    }
}

pub struct WriteService;

impl Default for WriteService {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteService {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(&self, intents: Vec<MutationIntent>) -> Result<Transaction, WriteError> {
        let id = format!("txn-{}", nonce());
        Ok(Transaction::new(id, intents))
    }

    pub fn plan_edits(
        &self,
        edits: &[ResolvedEdit],
        create: &[NewFile],
        files: &dyn FileSource,
        fingerprint: Fingerprint,
    ) -> Result<MutationPlan, WriteError> {
        for nf in create {
            if files.read(&nf.path).is_ok() {
                return Err(WriteError::EditFailed(format!(
                    "refusing to overwrite existing path '{}'; repository was not modified",
                    nf.path
                )));
            }
        }

        let mut diff = String::new();
        for nf in create {
            diff.push_str(&creation_diff(&nf.path, &nf.content));
        }

        let mut by_file: BTreeMap<&str, Vec<&ResolvedEdit>> = BTreeMap::new();
        for edit in edits {
            by_file.entry(edit.file.as_str()).or_default().push(edit);
        }

        let mut file_changes = Vec::new();

        for (path, file_edits) in by_file {
            let before = files.read(path)?;
            let mut current = before.clone();

            for edit in file_edits {
                current = apply_edit(path, &current, edit)?;
            }

            if current == before {
                continue;
            }

            file_changes.push(FileChange {
                path: path.to_string(),
                before: before.clone(),
                after: current.clone(),
            });
            diff.push_str(&unified_diff(path, &before, &current));
        }

        Ok(MutationPlan {
            edits: edits.to_vec(),
            create: create.to_vec(),
            file_changes,
            diff,
            fingerprint,
            status: TransactionStatus::Ready,
        })
    }

    pub fn apply(&mut self, transaction: &mut Transaction) -> Result<(), WriteError> {
        transaction.status = TransactionStatus::Applied;
        Ok(())
    }

    pub fn apply_plan(
        &self,
        plan: MutationPlan,
        intents: Vec<MutationIntent>,
        files: &mut dyn FileSource,
    ) -> Result<Transaction, WriteError> {
        let mut created: Vec<String> = Vec::new();
        let mut written: Vec<(String, String)> = Vec::new();

        for nf in &plan.create {
            if let Err(e) = files.write(&nf.path, &nf.content) {
                rollback_generation(&created, &written, files);
                return Err(e);
            }
            created.push(nf.path.clone());
        }

        for change in &plan.file_changes {
            if let Err(e) = files.write(&change.path, &change.after) {
                rollback_generation(&created, &written, files);
                return Err(e);
            }
            written.push((change.path.clone(), change.before.clone()));
        }

        for nf in &plan.create {
            let content = match files.read(&nf.path) {
                Ok(c) => c,
                Err(e) => {
                    rollback_generation(&created, &written, files);
                    return Err(e);
                }
            };
            if let Err(parse_err) = avalanche_source::parse(&content) {
                rollback_generation(&created, &written, files);
                return Err(WriteError::ValidationFailed(format!(
                    "{} produced unparseable Nix after creation: {}; repository was rolled back",
                    nf.path, parse_err
                )));
            }
        }

        for change in &plan.file_changes {
            let content = match files.read(&change.path) {
                Ok(c) => c,
                Err(e) => {
                    rollback_generation(&created, &written, files);
                    return Err(e);
                }
            };
            if let Err(parse_err) = avalanche_source::parse(&content) {
                rollback_generation(&created, &written, files);
                return Err(WriteError::ValidationFailed(format!(
                    "{} produced unparseable Nix after edit: {}; repository was rolled back",
                    change.path, parse_err
                )));
            }
        }

        let mut file_changes: Vec<FileChange> = plan
            .create
            .iter()
            .map(|nf| FileChange {
                path: nf.path.clone(),
                before: String::new(),
                after: nf.content.clone(),
            })
            .collect();
        file_changes.extend(plan.file_changes);

        Ok(Transaction {
            id: format!("txn-{}", nonce()),
            intents,
            file_changes,
            validation: Some(ValidationReport {
                tier: ValidationTier::Structural,
                findings: Vec::new(),
                passed: true,
            }),
            diff: Some(plan.diff),
            status: TransactionStatus::Applied,
        })
    }

    pub fn plan_generation(
        &self,
        create: &[NewFile],
        edits: &[ResolvedEdit],
        files: &dyn FileSource,
        fingerprint: Fingerprint,
    ) -> Result<GenerationPlan, WriteError> {
        for nf in create {
            if files.read(&nf.path).is_ok() {
                return Err(WriteError::EditFailed(format!(
                    "refusing to overwrite existing path '{}'; repository was not modified",
                    nf.path
                )));
            }
        }

        let mut diff = String::new();
        for nf in create {
            diff.push_str(&creation_diff(&nf.path, &nf.content));
        }

        let mut by_file: BTreeMap<&str, Vec<&ResolvedEdit>> = BTreeMap::new();
        for edit in edits {
            by_file.entry(edit.file.as_str()).or_default().push(edit);
        }
        let mut modified = Vec::new();
        for (path, file_edits) in by_file {
            let before = files.read(path)?;
            let mut current = before.clone();
            for edit in file_edits {
                current = apply_edit(path, &current, edit)?;
            }
            if current == before {
                continue;
            }
            modified.push(FileChange {
                path: path.to_string(),
                before: before.clone(),
                after: current.clone(),
            });
            diff.push_str(&unified_diff(path, &before, &current));
        }

        Ok(GenerationPlan {
            create: create.to_vec(),
            edits: edits.to_vec(),
            modified,
            diff,
            fingerprint,
            status: TransactionStatus::Ready,
        })
    }

    pub fn apply_generation(
        &self,
        plan: GenerationPlan,
        intents: Vec<MutationIntent>,
        files: &mut dyn FileSource,
    ) -> Result<Transaction, WriteError> {
        let mut created: Vec<String> = Vec::new();
        let mut backups: Vec<(String, String)> = Vec::new();

        for nf in &plan.create {
            if let Err(e) = files.write(&nf.path, &nf.content) {
                rollback_generation(&created, &backups, files);
                return Err(e);
            }
            created.push(nf.path.clone());
        }

        let mut by_file: BTreeMap<&str, Vec<&ResolvedEdit>> = BTreeMap::new();
        for edit in &plan.edits {
            by_file.entry(edit.file.as_str()).or_default().push(edit);
        }
        for (path, file_edits) in by_file {
            let before = match files.read(path) {
                Ok(c) => c,
                Err(e) => {
                    rollback_generation(&created, &backups, files);
                    return Err(e);
                }
            };
            let mut current = before.clone();
            let mut apply_failed = None;
            for edit in file_edits {
                match apply_edit(path, &current, edit) {
                    Ok(c) => current = c,
                    Err(e) => {
                        apply_failed = Some(e);
                        break;
                    }
                }
            }
            if let Some(e) = apply_failed {
                rollback_generation(&created, &backups, files);
                return Err(e);
            }
            if current == before {
                continue;
            }
            if let Err(e) = files.write(path, &current) {
                rollback_generation(&created, &backups, files);
                return Err(e);
            }
            backups.push((path.to_string(), before));
        }

        for nf in &plan.create {
            let content = match files.read(&nf.path) {
                Ok(c) => c,
                Err(e) => {
                    rollback_generation(&created, &backups, files);
                    return Err(e);
                }
            };
            if let Err(parse_err) = avalanche_source::parse(&content) {
                rollback_generation(&created, &backups, files);
                return Err(WriteError::ValidationFailed(format!(
                    "{} produced unparseable Nix after creation: {}; repository was rolled back",
                    nf.path, parse_err
                )));
            }
        }
        for change in &plan.modified {
            let content = match files.read(&change.path) {
                Ok(c) => c,
                Err(e) => {
                    rollback_generation(&created, &backups, files);
                    return Err(e);
                }
            };
            if let Err(parse_err) = avalanche_source::parse(&content) {
                rollback_generation(&created, &backups, files);
                return Err(WriteError::ValidationFailed(format!(
                    "{} produced unparseable Nix after edit: {}; repository was rolled back",
                    change.path, parse_err
                )));
            }
        }

        let mut file_changes: Vec<FileChange> = plan
            .create
            .iter()
            .map(|nf| FileChange {
                path: nf.path.clone(),
                before: String::new(),
                after: nf.content.clone(),
            })
            .collect();
        file_changes.extend(plan.modified.clone());

        Ok(Transaction {
            id: format!("txn-{}", nonce()),
            intents,
            file_changes,
            validation: Some(ValidationReport {
                tier: ValidationTier::Structural,
                findings: Vec::new(),
                passed: true,
            }),
            diff: Some(plan.diff),
            status: TransactionStatus::Applied,
        })
    }
}

fn rollback_generation(
    created: &[String],
    backups: &[(String, String)],
    files: &mut dyn FileSource,
) {
    for (path, before) in backups.iter().rev() {
        let _ = files.write(path, before);
    }
    for path in created.iter().rev() {
        let _ = files.remove(path);
    }
}

fn creation_diff(path: &str, content: &str) -> String {
    let n = content.lines().count();
    let mut out = format!("--- /dev/null\n+++ b/{path}\n@@ -0,0 +1,{n} @@\n");
    for line in content.lines() {
        out.push_str(&format!("+{line}\n"));
    }
    out
}

fn apply_edit(path: &str, source: &str, edit: &ResolvedEdit) -> Result<String, WriteError> {
    let patch = match edit.kind {
        EditKind::ReplaceValue => {
            let value = edit.value.as_ref().ok_or_else(|| {
                WriteError::EditFailed(format!("{path}: replace requires a value"))
            })?;
            let expr = render_expr(value, edit.render, &edit.option_path)?;
            generate_value_replacement_patch(path, source, &edit.option_path, &expr)
        }
        EditKind::InsertBinding => {
            let value = edit.value.as_ref().ok_or_else(|| {
                WriteError::EditFailed(format!("{path}: insert requires a value"))
            })?;
            let expr = render_expr(value, edit.render, &edit.option_path)?;
            generate_binding_insertion_patch(path, source, &edit.option_path, &expr)
        }
        EditKind::RemoveBinding => generate_binding_removal_patch(path, source, &edit.option_path),
    }
    .map_err(|e| match e {
        PatchError::AssignmentNotFound { path: p } => WriteError::EditFailed(format!(
            "{p} is not defined in {path}; repository was not modified"
        )),
        other => WriteError::EditFailed(format!("{path}: {other}")),
    })?;

    apply_patch(source, &patch).map_err(|e| WriteError::EditFailed(format!("{path}: {e}")))
}

fn render_expr(
    value: &serde_json::Value,
    render: RenderHint,
    option_path: &str,
) -> Result<Expr, WriteError> {
    let base = value_to_expr(value, option_path)?;
    Ok(match render {
        RenderHint::Literal => base,
        RenderHint::MkDefault => Expr::Apply {
            func: Box::new(Expr::AttrPath(vec!["lib".into(), "mkDefault".into()])),
            arg: Box::new(base),
        },
        RenderHint::MkForce => Expr::Apply {
            func: Box::new(Expr::AttrPath(vec!["lib".into(), "mkForce".into()])),
            arg: Box::new(base),
        },
    })
}

fn value_to_expr(value: &serde_json::Value, option_path: &str) -> Result<Expr, WriteError> {
    let unsupported = |reason: &str| WriteError::UnsupportedValue {
        option_path: option_path.to_string(),
        reason: reason.to_string(),
    };
    match value {
        serde_json::Value::Bool(b) => Ok(Expr::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Expr::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Expr::Float(f))
            } else {
                Err(unsupported("number out of range"))
            }
        }
        serde_json::Value::String(s) => Ok(Expr::Str(s.clone())),
        serde_json::Value::Array(items) => {
            let mut exprs = Vec::with_capacity(items.len());
            for item in items {
                exprs.push(value_to_expr(item, option_path)?);
            }
            Ok(Expr::List(exprs))
        }
        serde_json::Value::Object(obj) => {
            if let Some(ty) = obj.get("__nix_type__").and_then(|v| v.as_str()) {
                let val = obj.get("value").and_then(|v| v.as_str());
                return match (ty, val) {
                    ("path", Some(p)) => Ok(Expr::Path(p.to_string())),
                    ("attrpath", Some(a)) => {
                        Ok(Expr::AttrPath(a.split('.').map(String::from).collect()))
                    }
                    ("ident", Some(i)) => Ok(Expr::Ident(i.to_string())),
                    _ => Err(unsupported(&format!("nix type '{ty}'"))),
                };
            }
            let span = SourceSpan::new(
                Position { line: 1, column: 1 },
                Position { line: 1, column: 1 },
            );
            let mut bindings = Vec::with_capacity(obj.len());
            for (key, val) in obj {
                bindings.push(Binding {
                    path: vec![key.clone()],
                    value: Some(value_to_expr(val, option_path)?),
                    inherit: false,
                    span,
                });
            }
            Ok(Expr::Attrs(bindings))
        }
        other => Err(unsupported(&format!(
            "cannot render {} as a Nix literal",
            other_type_name(other)
        ))),
    }
}

fn other_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn unified_diff(path: &str, before: &str, after: &str) -> String {
    let a: Vec<&str> = before.lines().collect();
    let b: Vec<&str> = after.lines().collect();
    let lcs = lcs_table(&a, &b);

    let mut ops: Vec<DiffOp> = Vec::new();
    let (mut i, mut j) = (a.len(), b.len());
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            ops.push(DiffOp::Equal(a[i - 1]));
            i -= 1;
            j -= 1;
        } else if lcs[i - 1][j] >= lcs[i][j - 1] {
            ops.push(DiffOp::Delete(a[i - 1]));
            i -= 1;
        } else {
            ops.push(DiffOp::Insert(b[j - 1]));
            j -= 1;
        }
    }
    while i > 0 {
        ops.push(DiffOp::Delete(a[i - 1]));
        i -= 1;
    }
    while j > 0 {
        ops.push(DiffOp::Insert(b[j - 1]));
        j -= 1;
    }
    ops.reverse();

    let mut out = String::new();
    out.push_str(&format!("--- a/{path}\n+++ b/{path}\n"));

    let ctx = 3usize;
    let n = ops.len();
    let mut idx = 0usize;
    while idx < n {
        if matches!(ops[idx], DiffOp::Equal(_)) {
            idx += 1;
            continue;
        }
        let start = idx.saturating_sub(ctx);
        let mut end = idx;
        let mut last_change = idx;
        while end < n {
            if !matches!(ops[end], DiffOp::Equal(_)) {
                last_change = end;
            }
            if end > last_change + ctx {
                break;
            }
            end += 1;
        }
        end = end.min(n);

        let mut a_start = 0usize;
        let mut b_start = 0usize;
        for (k, op) in ops.iter().enumerate() {
            if k >= start {
                break;
            }
            match op {
                DiffOp::Equal(_) => {
                    a_start += 1;
                    b_start += 1;
                }
                DiffOp::Delete(_) => a_start += 1,
                DiffOp::Insert(_) => b_start += 1,
            }
        }
        let mut a_len = 0usize;
        let mut b_len = 0usize;
        for op in &ops[start..end] {
            match op {
                DiffOp::Equal(_) => {
                    a_len += 1;
                    b_len += 1;
                }
                DiffOp::Delete(_) => a_len += 1,
                DiffOp::Insert(_) => b_len += 1,
            }
        }

        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            a_start + 1,
            a_len,
            b_start + 1,
            b_len
        ));
        for op in &ops[start..end] {
            match op {
                DiffOp::Equal(l) => out.push_str(&format!(" {l}\n")),
                DiffOp::Delete(l) => out.push_str(&format!("-{l}\n")),
                DiffOp::Insert(l) => out.push_str(&format!("+{l}\n")),
            }
        }
        idx = end;
    }

    out
}

enum DiffOp<'a> {
    Equal(&'a str),
    Delete(&'a str),
    Insert(&'a str),
}

fn lcs_table(a: &[&str], b: &[&str]) -> Vec<Vec<usize>> {
    let mut table = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            table[i][j] = if a[i] == b[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    table
}

fn nonce() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}{:x}", now.as_secs(), now.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_model::source::{Position, SourceSpan};
    use avalanche_model::{Layer, Scope};
    use avalanche_source::parse;
    use std::collections::HashMap;

    struct MapFileSource {
        files: HashMap<String, String>,
    }

    impl FileSource for MapFileSource {
        fn read(&self, rel_path: &str) -> Result<String, WriteError> {
            self.files
                .get(rel_path)
                .cloned()
                .ok_or_else(|| WriteError::Io {
                    path: rel_path.to_string(),
                    reason: "not found in map".to_string(),
                })
        }

        fn write(&mut self, rel_path: &str, content: &str) -> Result<(), WriteError> {
            self.files.insert(rel_path.to_string(), content.to_string());
            Ok(())
        }

        fn remove(&mut self, rel_path: &str) -> Result<(), WriteError> {
            self.files.remove(rel_path);
            Ok(())
        }
    }

    fn edit(
        file: &str,
        option_path: &str,
        kind: EditKind,
        value: Option<serde_json::Value>,
    ) -> ResolvedEdit {
        ResolvedEdit {
            file: file.to_string(),
            option_path: option_path.to_string(),
            kind,
            value,
            render: RenderHint::Literal,
            layer: Layer::Capability,
            scope: Scope::System,
            host: None,
            user: None,
            profile: None,
            reason: "test".to_string(),
        }
    }

    fn fp() -> Fingerprint {
        Fingerprint {
            git_head: None,
            working_tree_hash: None,
            flake_lock_hash: None,
            schema_version: None,
            index_schema_version: 1,
        }
    }

    #[test]
    fn value_to_expr_literals() {
        assert_eq!(
            value_to_expr(&serde_json::json!(true), "x").unwrap(),
            Expr::Bool(true)
        );
        assert_eq!(
            value_to_expr(&serde_json::json!(7), "x").unwrap(),
            Expr::Int(7)
        );
        assert_eq!(
            value_to_expr(&serde_json::json!("hi"), "x").unwrap(),
            Expr::Str("hi".into())
        );
        assert_eq!(
            value_to_expr(&serde_json::json!([1, 2]), "x").unwrap(),
            Expr::List(vec![Expr::Int(1), Expr::Int(2)])
        );
    }

    #[test]
    fn value_to_expr_renders_plain_object_as_attrs() {
        let expr = value_to_expr(&serde_json::json!({ "a": 1 }), "svc.enable").unwrap();
        assert_eq!(
            expr,
            Expr::Attrs(vec![Binding {
                path: vec!["a".to_string()],
                value: Some(Expr::Int(1)),
                inherit: false,
                span: SourceSpan::new(
                    Position { line: 1, column: 1 },
                    Position { line: 1, column: 1 }
                ),
            }])
        );
    }

    #[test]
    fn value_to_expr_rejects_unknown_nix_type() {
        let err = value_to_expr(
            &serde_json::json!({ "__nix_type__": "bogus", "value": "x" }),
            "svc.enable",
        )
        .unwrap_err();
        assert!(matches!(err, WriteError::UnsupportedValue { .. }));
    }

    #[test]
    fn mk_default_renders_and_parses() {
        let expr = render_expr(&serde_json::json!(true), RenderHint::MkDefault, "x").unwrap();
        let text = avalanche_source::format_expr(&expr);
        assert_eq!(text, "lib.mkDefault true");
        let parsed = parse(&format!("{{ enable = {text}; }}")).unwrap();
        assert!(format_expr_roundtrip(&parsed));
    }

    fn format_expr_roundtrip(file: &avalanche_source::NixFile) -> bool {
        let rendered = avalanche_source::format_expr(&file.expr);
        rendered.contains("lib.mkDefault true")
    }

    #[test]
    fn plan_replace_value_produces_change_and_diff() {
        let mut files = HashMap::new();
        files.insert(
            "m.nix".to_string(),
            "{\n  services.pipewire.enable = true;\n}\n".to_string(),
        );
        let src = MapFileSource { files };
        let edits = vec![edit(
            "m.nix",
            "services.pipewire.enable",
            EditKind::ReplaceValue,
            Some(serde_json::json!(false)),
        )];
        let plan = WriteService::new()
            .plan_edits(&edits, &[], &src, fp())
            .unwrap();
        assert_eq!(plan.file_changes.len(), 1);
        assert!(plan.file_changes[0].after.contains("false"));
        assert!(plan.diff.contains("-  services.pipewire.enable = true;"));
        assert!(plan.diff.contains("+  services.pipewire.enable = false;"));
        assert_eq!(plan.status, TransactionStatus::Ready);
    }

    #[test]
    fn plan_insert_binding_adds_new_option() {
        let mut files = HashMap::new();
        files.insert(
            "m.nix".to_string(),
            "{\n  existing = true;\n}\n".to_string(),
        );
        let src = MapFileSource { files };
        let edits = vec![edit(
            "m.nix",
            "programs.git.enable",
            EditKind::InsertBinding,
            Some(serde_json::json!(true)),
        )];
        let plan = WriteService::new()
            .plan_edits(&edits, &[], &src, fp())
            .unwrap();
        assert_eq!(plan.file_changes.len(), 1);
        assert!(plan.file_changes[0]
            .after
            .contains("programs.git.enable = true;"));
    }

    #[test]
    fn plan_remove_binding_deletes_override() {
        let mut files = HashMap::new();
        files.insert(
            "hosts/pc/default.nix".to_string(),
            "{\n  services.openssh.enable = true;\n}\n".to_string(),
        );
        let src = MapFileSource { files };
        let edits = vec![edit(
            "hosts/pc/default.nix",
            "services.openssh.enable",
            EditKind::RemoveBinding,
            None,
        )];
        let plan = WriteService::new()
            .plan_edits(&edits, &[], &src, fp())
            .unwrap();
        assert_eq!(plan.file_changes.len(), 1);
        assert!(!plan.file_changes[0]
            .after
            .contains("services.openssh.enable = true;"));
        assert!(!plan.file_changes[0].after.contains("= false"));
    }

    #[test]
    fn plan_noop_when_value_unchanged() {
        let mut files = HashMap::new();
        files.insert("m.nix".to_string(), "{\n  enable = true;\n}\n".to_string());
        let src = MapFileSource { files };
        let edits = vec![edit(
            "m.nix",
            "enable",
            EditKind::ReplaceValue,
            Some(serde_json::json!(true)),
        )];
        let plan = WriteService::new()
            .plan_edits(&edits, &[], &src, fp())
            .unwrap();
        assert!(plan.file_changes.is_empty());
        assert!(plan.diff.is_empty());
    }

    #[test]
    fn plan_missing_file_is_io_error() {
        let src = MapFileSource {
            files: HashMap::new(),
        };
        let edits = vec![edit(
            "gone.nix",
            "a.b",
            EditKind::InsertBinding,
            Some(serde_json::json!(1)),
        )];
        let err = WriteService::new()
            .plan_edits(&edits, &[], &src, fp())
            .unwrap_err();
        assert!(matches!(err, WriteError::Io { .. }));
    }

    #[test]
    fn unified_diff_marks_context_and_changes() {
        let d = unified_diff("f", "a\nb\nc\n", "a\nB\nc\n");
        assert!(d.contains("@@"));
        assert!(d.contains("-b"));
        assert!(d.contains("+B"));
        assert!(d.contains(" a"));
    }

    #[test]
    fn span_helpers_unused_guard() {
        let _ = SourceSpan::new(
            Position { line: 1, column: 1 },
            Position { line: 1, column: 2 },
        );
    }

    #[test]
    fn generation_creates_file_and_modifies_existing() {
        let mut files = HashMap::new();
        files.insert("m.nix".to_string(), "{\n  enable = true;\n}\n".to_string());
        let src = MapFileSource {
            files: files.clone(),
        };
        let create = vec![NewFile {
            path: "new.nix".to_string(),
            content: "{\n  x = 1;\n}\n".to_string(),
        }];
        let edits = vec![edit(
            "m.nix",
            "enable",
            EditKind::ReplaceValue,
            Some(serde_json::json!(false)),
        )];
        let plan = WriteService::new()
            .plan_generation(&create, &edits, &src, fp())
            .unwrap();

        let mut target = MapFileSource { files };
        let txn = WriteService::new()
            .apply_generation(plan, vec![], &mut target)
            .unwrap();

        assert_eq!(txn.status, TransactionStatus::Applied);
        assert_eq!(target.files["new.nix"], "{\n  x = 1;\n}\n");
        assert!(target.files["m.nix"].contains("enable = false;"));
        assert!(txn
            .file_changes
            .iter()
            .any(|c| c.path == "new.nix" && c.before.is_empty()));
        assert!(txn.diff.as_deref().unwrap().contains("--- /dev/null"));
    }

    #[test]
    fn generation_refuses_to_overwrite_existing_path() {
        let mut files = HashMap::new();
        files.insert("dup.nix".to_string(), "{\n  a = 1;\n}\n".to_string());
        let src = MapFileSource { files };
        let create = vec![NewFile {
            path: "dup.nix".to_string(),
            content: "x".to_string(),
        }];
        let err = WriteService::new()
            .plan_generation(&create, &[], &src, fp())
            .unwrap_err();
        assert!(matches!(err, WriteError::EditFailed(_)));
    }

    #[test]
    fn generation_plan_failure_writes_nothing() {
        let mut files = HashMap::new();
        files.insert("m.nix".to_string(), "{\n  enable = true;\n}\n".to_string());
        let src = MapFileSource {
            files: files.clone(),
        };
        let create = vec![NewFile {
            path: "new.nix".to_string(),
            content: "{\n  ok = true;\n}\n".to_string(),
        }];
        let edits = vec![edit(
            "m.nix",
            "nope.nope",
            EditKind::ReplaceValue,
            Some(serde_json::json!(1)),
        )];
        let err = WriteService::new()
            .plan_generation(&create, &edits, &src, fp())
            .unwrap_err();
        assert!(matches!(err, WriteError::EditFailed(_)));
        let target = MapFileSource { files };
        assert!(!target.files.contains_key("new.nix"));
    }

    #[test]
    fn generation_rolls_back_created_files_on_parse_failure() {
        let mut files = HashMap::new();
        files.insert("m.nix".to_string(), "{\n  enable = true;\n}\n".to_string());
        let src = MapFileSource {
            files: files.clone(),
        };
        let create = vec![NewFile {
            path: "bad.nix".to_string(),
            content: "{ a = ;; }\n".to_string(),
        }];
        let edits = vec![edit(
            "m.nix",
            "enable",
            EditKind::ReplaceValue,
            Some(serde_json::json!(false)),
        )];
        let plan = WriteService::new()
            .plan_generation(&create, &edits, &src, fp())
            .unwrap();

        let mut target = MapFileSource { files };
        let err = WriteService::new()
            .apply_generation(plan, vec![], &mut target)
            .unwrap_err();
        assert!(matches!(err, WriteError::ValidationFailed(_)));
        assert!(!target.files.contains_key("bad.nix"));
        assert!(target.files["m.nix"].contains("enable = true;"));
        assert!(!target.files["m.nix"].contains("enable = false"));
    }

    struct FlakyFileSource {
        files: HashMap<String, String>,
        fail_paths: std::collections::HashSet<String>,
    }

    impl FileSource for FlakyFileSource {
        fn read(&self, rel_path: &str) -> Result<String, WriteError> {
            self.files
                .get(rel_path)
                .cloned()
                .ok_or_else(|| WriteError::Io {
                    path: rel_path.to_string(),
                    reason: "not found in map".to_string(),
                })
        }

        fn write(&mut self, rel_path: &str, content: &str) -> Result<(), WriteError> {
            if self.fail_paths.contains(rel_path) {
                return Err(WriteError::Io {
                    path: rel_path.to_string(),
                    reason: "injected write failure".to_string(),
                });
            }
            self.files.insert(rel_path.to_string(), content.to_string());
            Ok(())
        }

        fn remove(&mut self, rel_path: &str) -> Result<(), WriteError> {
            self.files.remove(rel_path);
            Ok(())
        }
    }

    #[test]
    fn apply_plan_writes_and_returns_applied_transaction() {
        let mut files = HashMap::new();
        files.insert(
            "m.nix".to_string(),
            "{\n  services.pipewire.enable = true;\n}\n".to_string(),
        );
        let src = MapFileSource {
            files: files.clone(),
        };
        let edits = vec![edit(
            "m.nix",
            "services.pipewire.enable",
            EditKind::ReplaceValue,
            Some(serde_json::json!(false)),
        )];
        let plan = WriteService::new()
            .plan_edits(&edits, &[], &src, fp())
            .unwrap();

        let mut target = MapFileSource { files };
        let txn = WriteService::new()
            .apply_plan(plan, vec![], &mut target)
            .unwrap();

        assert_eq!(txn.status, TransactionStatus::Applied);
        assert!(txn.diff.is_some());
        assert!(txn.validation.map(|v| v.passed).unwrap_or(false));
        assert!(target.files["m.nix"].contains("services.pipewire.enable = false;"));
    }

    #[test]
    fn apply_plan_rolls_back_on_write_failure() {
        let mut files = HashMap::new();
        files.insert("a.nix".to_string(), "{\n  x = 1;\n}\n".to_string());
        files.insert("b.nix".to_string(), "{\n  y = 2;\n}\n".to_string());
        let src = MapFileSource {
            files: files.clone(),
        };
        let edits = vec![
            edit(
                "a.nix",
                "x",
                EditKind::ReplaceValue,
                Some(serde_json::json!(9)),
            ),
            edit(
                "b.nix",
                "y",
                EditKind::ReplaceValue,
                Some(serde_json::json!(8)),
            ),
        ];
        let plan = WriteService::new()
            .plan_edits(&edits, &[], &src, fp())
            .unwrap();
        assert_eq!(plan.file_changes.len(), 2);

        let fail: std::collections::HashSet<String> = ["b.nix".to_string()].into_iter().collect();
        let mut flaky = FlakyFileSource {
            files,
            fail_paths: fail,
        };
        let err = WriteService::new()
            .apply_plan(plan, vec![], &mut flaky)
            .unwrap_err();
        assert!(matches!(err, WriteError::Io { .. }));
        assert_eq!(flaky.files["a.nix"], "{\n  x = 1;\n}\n");
        assert_eq!(flaky.files["b.nix"], "{\n  y = 2;\n}\n");
    }
}
