use std::collections::HashMap;
use url::Url;

use crate::types::*;

pub struct KlyronLsp {
    pub documents: HashMap<Url, Document>,
    pub diagnostics: HashMap<Url, Vec<Diagnostic>>,
    pub completions: CompletionEngine,
    pub symbols: SymbolIndex,
    initialized: bool,
    #[allow(dead_code)]
    client_capabilities: Option<ClientCapabilities>,
}

impl KlyronLsp {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            diagnostics: HashMap::new(),
            completions: CompletionEngine::new(),
            symbols: SymbolIndex::new(),
            initialized: false,
            client_capabilities: None,
        }
    }

    pub fn initialize(params: InitializeParams) -> InitializeResult {
        let _ = params;
        InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Incremental),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), "/".into()]),
                }),
                definition_provider: Some(true),
                references_provider: Some(true),
                hover_provider: Some(true),
                document_symbol_provider: Some(true),
                workspace_symbol_provider: Some(true),
                code_action_provider: Some(true),
                rename_provider: Some(true),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".into(), ",".into()]),
                }),
            },
        }
    }

    pub fn initialized(&mut self) {
        self.initialized = true;
    }

    pub fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let doc = Document {
            uri: params.text_document.uri.clone(),
            text: params.text_document.text,
            version: params.text_document.version,
            language_id: params.text_document.language_id,
        };
        self.documents.insert(doc.uri.clone(), doc);
    }

    pub fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(doc) = self.documents.get_mut(&uri) {
            doc.version = params.text_document.version;
            for change in params.content_changes {
                if let Some(range) = change.range {
                    let start = offset_from_position(&doc.text, range.start);
                    let end = offset_from_position(&doc.text, range.end);
                    doc.text.replace_range(start..end, &change.text);
                } else {
                    doc.text = change.text;
                }
            }
        }
    }

    pub fn did_close(&mut self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
        self.diagnostics.remove(&params.text_document.uri);
    }

    pub fn did_save(&mut self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            if let Some(doc) = self.documents.get_mut(&params.text_document.uri) {
                doc.text = text;
            }
        }
    }

    pub fn completion(&self, _params: CompletionParams) -> CompletionResponse {
        self.completions.items.clone()
    }

    pub fn goto_definition(&self, _params: GotoDefinitionParams) -> Option<Location> {
        None
    }

    pub fn find_references(&self, _params: ReferenceParams) -> Vec<Location> {
        Vec::new()
    }

    pub fn hover(&self, _params: HoverParams) -> Option<Hover> {
        None
    }

    pub fn document_symbols(&self, _params: DocumentSymbolParams) -> Vec<DocumentSymbol> {
        Vec::new()
    }

    pub fn code_action(&self, _params: CodeActionParams) -> Vec<CodeAction> {
        Vec::new()
    }

    pub fn rename(&self, _params: RenameParams) -> Option<WorkspaceEdit> {
        None
    }

    pub fn signature_help(&self, _params: SignatureHelpParams) -> Option<SignatureHelp> {
        None
    }
}

impl Default for KlyronLsp {
    fn default() -> Self {
        Self::new()
    }
}

fn offset_from_position(text: &str, pos: Position) -> usize {
    let mut offset = 0;
    for _ in 0..pos.line {
        let nl = text[offset..].find('\n');
        match nl {
            Some(n) => offset += n + 1,
            None => return text.len(),
        }
    }
    (offset + pos.character as usize).min(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_server() {
        let server = KlyronLsp::new();
        assert!(server.documents.is_empty());
        assert!(server.diagnostics.is_empty());
        assert!(!server.initialized);
    }

    #[test]
    fn test_initialize() {
        let params = InitializeParams {
            process_id: None,
            root_uri: None,
            capabilities: ClientCapabilities::default(),
        };
        let result = KlyronLsp::initialize(params);
        assert!(result.capabilities.definition_provider == Some(true));
    }

    #[test]
    fn test_did_open() {
        let mut server = KlyronLsp::new();
        let uri = Url::parse("file:///test.kly").unwrap();
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "klyron".into(),
                version: 1,
                text: "hello".into(),
            },
        };
        server.did_open(params);
        assert!(server.documents.contains_key(&uri));
        assert_eq!(server.documents[&uri].text, "hello");
    }

    #[test]
    fn test_did_close() {
        let mut server = KlyronLsp::new();
        let uri = Url::parse("file:///test.kly").unwrap();
        server.documents.insert(
            uri.clone(),
            Document {
                uri: uri.clone(),
                text: "hello".into(),
                version: 1,
                language_id: "klyron".into(),
            },
        );
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
        };
        server.did_close(params);
        assert!(!server.documents.contains_key(&uri));
    }

    #[test]
    fn test_completion() {
        let server = KlyronLsp::new();
        let uri = Url::parse("file:///test.kly").unwrap();
        let params = CompletionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position { line: 0, character: 0 },
        };
        let result = server.completion(params);
        assert!(result.is_empty());
    }

    #[test]
    fn test_document_symbols() {
        let server = KlyronLsp::new();
        let uri = Url::parse("file:///test.kly").unwrap();
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
        };
        let result = server.document_symbols(params);
        assert!(result.is_empty());
    }
}
