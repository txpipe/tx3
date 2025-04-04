use dashmap::DashMap;
use ropey::Rope;
use serde_json::{Value, Map};
use std::str::FromStr;
use tower::ServiceBuilder;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tx3_lang::Protocol;

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
                execute_command_provider: Some(
                    ExecuteCommandOptions {
                        commands: vec!["generate-tir".to_string()],
                        work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None }
                    }
                ),
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

    // TODO: Add error handling and improve
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        
        fn make_symbol(
            name: String,
            detail: String,
            kind: SymbolKind,
            range: Range,
            children: Option<Vec<DocumentSymbol>>,
        ) -> DocumentSymbol {
            #[allow(deprecated)]
            DocumentSymbol {
                name,
                detail: Some(detail),
                kind,
                range: range,
                selection_range: range,
                children: children,
                tags: Default::default(),
                deprecated: Default::default(),
            }
        }

        let mut symbols: Vec<DocumentSymbol> = Vec::new();
        let uri = &params.text_document.uri;
        let document = self.documents.get(uri);
        if let Some(document) = document {
            let text = document.value().to_string();
            let ast = tx3_lang::parsing::parse_string(text.as_str());
            if ast.is_ok() {
                let ast = ast.unwrap();
                for party in ast.parties {
                    symbols.push(make_symbol(
                        party.name.clone(),
                        "Party".to_string(),
                        SymbolKind::OBJECT,
                        span_to_lsp_range(document.value(), &party.span),
                        None,
                    ));
                }
                for policy in ast.policies {
                    symbols.push(make_symbol(
                        policy.name.clone(),
                        "Policy".to_string(),
                        SymbolKind::KEY,
                        span_to_lsp_range(document.value(), &policy.span),
                        None,
                    ));
                }
                for tx in ast.txs {
                    let mut children: Vec<DocumentSymbol> = Vec::new();
                    for parameter in tx.parameters.parameters {
                        children.push(make_symbol(
                            parameter.name.clone(),
                            format!("Parameter<{:?}>", parameter.r#type),
                            SymbolKind::FIELD,
                            span_to_lsp_range(document.value(), &tx.parameters.span),
                            None,
                        ));
                    }
                    for input in tx.inputs {
                        children.push(make_symbol(
                            input.name.clone(),
                            "Input".to_string(),
                            SymbolKind::OBJECT,
                            span_to_lsp_range(document.value(), &input.span),
                            None,
                        ));
                    }
                    for output in tx.outputs {
                        children.push(make_symbol(
                            output.name.unwrap_or_else(|| {"output"}.to_string()),
                            "Output".to_string(),
                            SymbolKind::OBJECT,
                            span_to_lsp_range(document.value(), &output.span),
                            None,
                        ));
                    }
                    symbols.push(make_symbol(
                        tx.name.clone(),
                        "Tx".to_string(),
                        SymbolKind::METHOD,
                        span_to_lsp_range(document.value(), &tx.span),
                        Some(children),
                    ));
                }
            }
        }
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn symbol(&self, _: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        // Return empty workspace symbols list for now
        Ok(Some(vec![]))
    }

    async fn symbol_resolve(&self, params: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        dbg!(&params);
        Ok(params)
    }

    // TODO: Add error handling
    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        if params.command == "generate-tir" {
            let uri = Url::from_str(params.arguments[0].as_str().unwrap());
            let document = self.documents.get(&uri.unwrap());
            if let Some(document) = document {
                let protocol = Protocol::from_string(document.value().to_string()).load().unwrap();
                let prototx = protocol.new_tx(params.arguments[1].as_str().unwrap()).unwrap();

                let mut response = Map::new();
                response.insert("tir".to_string(), Value::String(hex::encode(prototx.ir_bytes())));

                let mut params = Map::new();
                prototx.find_params().iter().for_each(|param| {
                    params.insert(param.0.to_string(), serde_json::to_value(param.1).unwrap());
                });
                response.insert("parameters".to_string(), Value::Object(params));

                return Ok(Some(Value::Object(response)));
            }
        }
        Ok(None)
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
