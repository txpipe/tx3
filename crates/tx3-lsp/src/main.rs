use dashmap::DashMap;
use ropey::Rope;
use tower::ServiceBuilder;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: DashMap<Url, Rope>,
    //asts: DashMap<Url, tx3_lang::ast::Program>,
}

fn char_index_to_line_col(rope: &Rope, idx: usize) -> (usize, usize) {
    let line = rope.char_to_line(idx);
    let line_start = rope.line_to_char(line);
    let col = idx - line_start;
    (line, col)
}

fn span_to_lsp_range(rope: &Rope, loc: &tx3_lang::ast::Span) -> Range {
    let (start_line, start_col) = char_index_to_line_col(rope, loc.start);
    let (end_line, end_col) = char_index_to_line_col(rope, loc.end);
    let start = Position::new(start_line as u32, start_col as u32);
    let end = Position::new(end_line as u32, end_col as u32);
    Range::new(start, end)
}

fn parse_error_to_diagnostic(rope: &Rope, err: &tx3_lang::parsing::Error) -> Diagnostic {
    let range = span_to_lsp_range(rope, &err.span);
    let message = err.message.clone();
    let source = err.src.clone();

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(source),
        message,
        ..Default::default()
    }
}

fn analyze_error_to_diagnostic(rope: &Rope, err: &tx3_lang::analyzing::Error) -> Diagnostic {
    let range = span_to_lsp_range(rope, err.span());
    let message = err.to_string();
    let source = err.src().unwrap_or("tx3").to_string();

    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(source),
        message,
        ..Default::default()
    }
}

fn analyze_report_to_diagnostic(
    rope: &Rope,
    report: &tx3_lang::analyzing::AnalyzeReport,
) -> Vec<Diagnostic> {
    report
        .errors
        .iter()
        .map(|err| analyze_error_to_diagnostic(rope, err))
        .collect()
}

impl Backend {
    async fn process_document(&self, uri: Url, text: &str) -> Vec<Diagnostic> {
        let rope = Rope::from_str(text);
        self.documents.insert(uri.clone(), rope.clone());

        let ast = tx3_lang::parsing::parse_string(text);

        match ast {
            Ok(mut ast) => {
                let analysis = tx3_lang::analyzing::analyze(&mut ast);
                analyze_report_to_diagnostic(&rope, &analysis)
            }
            Err(e) => vec![parse_error_to_diagnostic(&rope, &e)],
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(Default::default()),
                definition_provider: Some(OneOf::Left(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Simple(true)),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),

                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "tx3-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Return empty completion list for now
        Ok(Some(CompletionResponse::Array(vec![])))
    }

    async fn goto_definition(
        &self,
        _: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // Return None for now, indicating no definition found
        Ok(None)
    }

    async fn references(&self, _: ReferenceParams) -> Result<Option<Vec<Location>>> {
        // Return empty references list for now
        Ok(Some(vec![]))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // Get the position where the user is hovering
        let position = params.text_document_position_params.position;

        // Here you would typically:
        // 1. Parse the document to identify the symbol at the hover position
        // 2. Look up information about that symbol
        // 3. Return a Hover object with the information

        // For now, let's return a simple example hover
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "This is a symbol hover example".to_string(),
            }),
            range: Some(Range {
                start: Position {
                    line: position.line,
                    character: position.character,
                },
                end: Position {
                    line: position.line,
                    character: position.character + 1,
                },
            }),
        }))
    }

    async fn document_symbol(
        &self,
        _: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        // Return empty symbols list for now
        Ok(Some(DocumentSymbolResponse::Flat(vec![])))
    }

    async fn symbol(&self, _: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        // Return empty workspace symbols list for now
        Ok(Some(vec![]))
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        let text = params.text_document.text.as_str();

        let diagnostics = self.process_document(uri.clone(), text).await;

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        let text = params
            .content_changes
            .first()
            .map(|x| x.text.as_str())
            .unwrap_or("");

        let diagnostics = self.process_document(uri.clone(), text).await;

        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: DashMap::new(),
    });

    // Create a logging middleware
    let service = ServiceBuilder::new()
        .map_request(|request| {
            dbg!(&request);
            request
        })
        .map_response(|response| {
            dbg!(&response);
            response
        })
        .service(service);

    Server::new(stdin, stdout, socket).serve(service).await;
}
