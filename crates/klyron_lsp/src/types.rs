use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub text: String,
    pub version: i32,
    pub language_id: String,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone)]
pub struct CompletionEngine {
    pub items: Vec<CompletionItem>,
    pub cache: HashMap<String, Vec<CompletionItem>>,
}

impl CompletionEngine {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cache: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolIndex {
    pub symbols_by_file: HashMap<Url, Vec<DocumentSymbol>>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self {
            symbols_by_file: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub uri: Url,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub struct Hover {
    pub contents: MarkupContent,
    pub range: Option<Range>,
}

#[derive(Debug, Clone)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub selection_range: Range,
    pub children: Vec<DocumentSymbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    File,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<String>,
    pub edit: Option<WorkspaceEdit>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceEdit {
    pub changes: HashMap<Url, Vec<TextEdit>>,
}

#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

#[derive(Debug, Clone)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInformation>,
    pub active_signature: Option<u32>,
    pub active_parameter: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct SignatureInformation {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInformation>,
}

#[derive(Debug, Clone)]
pub struct ParameterInformation {
    pub label: String,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    pub text_document_sync: Option<TextDocumentSyncCapability>,
    pub completion_provider: Option<CompletionOptions>,
    pub definition_provider: Option<bool>,
    pub references_provider: Option<bool>,
    pub hover_provider: Option<bool>,
    pub document_symbol_provider: Option<bool>,
    pub workspace_symbol_provider: Option<bool>,
    pub code_action_provider: Option<bool>,
    pub rename_provider: Option<bool>,
    pub signature_help_provider: Option<SignatureHelpOptions>,
}

#[derive(Debug, Clone)]
pub enum TextDocumentSyncCapability {
    None,
    Full,
    Incremental,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionOptions {
    pub trigger_characters: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct SignatureHelpOptions {
    pub trigger_characters: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct InitializeParams {
    pub process_id: Option<i32>,
    pub root_uri: Option<Url>,
    pub capabilities: ClientCapabilities,
}

#[derive(Debug, Clone)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Clone, Default)]
pub struct ClientCapabilities {
    pub text_document: Option<TextDocumentClientCapabilities>,
    pub workspace: Option<WorkspaceClientCapabilities>,
}

#[derive(Debug, Clone, Default)]
pub struct TextDocumentClientCapabilities;

#[derive(Debug, Clone, Default)]
pub struct WorkspaceClientCapabilities;

#[derive(Debug, Clone)]
pub struct DidOpenTextDocumentParams {
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Clone)]
pub struct TextDocumentItem {
    pub uri: Url,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Clone)]
pub struct TextDocumentContentChangeEvent {
    pub range: Option<Range>,
    pub range_length: Option<u32>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct DidCloseTextDocumentParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Clone)]
pub struct DidSaveTextDocumentParams {
    pub text_document: TextDocumentIdentifier,
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompletionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct GotoDefinitionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct ReferenceParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct HoverParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct DocumentSymbolParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Clone)]
pub struct CodeActionParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub struct RenameParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    pub new_name: String,
}

#[derive(Debug, Clone)]
pub struct SignatureHelpParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Clone)]
pub struct TextDocumentIdentifier {
    pub uri: Url,
}

#[derive(Debug, Clone)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: Url,
    pub version: i32,
}

pub type CompletionResponse = Vec<CompletionItem>;

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}
