use std::sync::Arc;

use basm::{
    lex::Advance,
    parse::{ParseError, Parser},
    Basm,
};
use basm_fmt::FmtContext;
use dashmap::DashMap;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        Diagnostic, DiagnosticOptions, DiagnosticServerCapabilities, DidChangeConfigurationParams,
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, DocumentDiagnosticParams, DocumentDiagnosticReport,
        DocumentDiagnosticReportResult, DocumentFormattingParams, DocumentSymbolParams,
        DocumentSymbolResponse, FormattingOptions, FullDocumentDiagnosticReport, InitializeParams,
        InitializeResult, InitializedParams, MessageType, OneOf, Position, PositionEncodingKind,
        Range, RelatedFullDocumentDiagnosticReport, SemanticTokens, SemanticTokensDeltaParams,
        SemanticTokensFullDeltaResult, SemanticTokensFullOptions, SemanticTokensLegend,
        SemanticTokensOptions, SemanticTokensParams, SemanticTokensRangeParams,
        SemanticTokensRangeResult, SemanticTokensResult, SemanticTokensServerCapabilities,
        ServerCapabilities, ServerInfo, TextDocumentSyncKind, TextEdit, Url,
    },
    Client, LanguageServer,
};

use semantic_tokens::{TOKEN_MODS, TOKEN_TYPES};

mod semantic_tokens;

// TODO: create error limits

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub forms: DashMap<Url, Document>,
}

fn line_range(ad: Advance) -> Range {
    let line = ad.line;
    let from = ad.span.from - ad.offset;
    let to = ad.span.to - ad.offset;
    Range {
        start: Position {
            line,
            character: from,
        },
        end: Position {
            line,
            character: to,
        },
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            forms: DashMap::new(),
        }
    }
    async fn info(&self, m: impl std::fmt::Display + Send) {
        tracing::info!("{m}");
        self.client.log_message(MessageType::INFO, m).await;
    }
    fn get_doc(&'_ self, uri: &Url) -> Result<dashmap::mapref::one::Ref<'_, Url, Document>> {
        self.forms.get(uri).map_or_else(unknown_uri, Ok)
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Document {
    src: Arc<str>,
    basm: Basm,
    lex: Vec<Advance>,
    errors: Vec<ParseError>,
}

impl Document {
    fn new(src: String) -> Self {
        let src = Arc::<str>::from(src);
        let (basm, errors, lex) = Parser::recorded(&src).parse();
        Self {
            src,
            basm,
            lex,
            errors,
        }
    }
    // TODO: add partial & delta semantic token changes
    fn diagnostics(&self) -> Vec<Diagnostic> {
        // TODO: merge errors
        self.errors
            .iter()
            .map(|e| {
                let range = line_range(e.advance());
                let message = e.to_string();
                Diagnostic {
                    range,
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }
    // TODO: add partial formatting
    fn formatting(&self, opts: FormattingOptions) -> Vec<TextEdit> {
        let fmt = basm_fmt::fmt(
            &self.basm,
            &self.lex,
            &self.src,
            &self.errors,
            &FmtContext {
                tab_size: opts.tab_size,
            },
        );
        fmt.into_iter()
            .map(|e| TextEdit {
                range: Range {
                    start: Position {
                        line: e.line,
                        character: e.span.from,
                    },
                    end: Position {
                        line: e.line,
                        character: e.span.to,
                    },
                },
                new_text: e.text,
            })
            .collect()
    }
}

fn capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncKind::FULL.into()),
        position_encoding: Some(PositionEncodingKind::UTF8),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
            DiagnosticOptions::default(),
        )),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: TOKEN_TYPES.to_vec(),
                    token_modifiers: TOKEN_MODS.into(),
                },
                range: Some(true),
                full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                ..Default::default()
            },
        )),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..Default::default()
    }
}

fn unknown_uri<T>() -> Result<T> {
    Err(tower_lsp::jsonrpc::Error::invalid_params(
        "unknown uri inputted",
    ))
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: capabilities(),
            server_info: Some(ServerInfo {
                name: "basm".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.info("cbnf-ls initialized").await;
    }
    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items: self.get_doc(&params.text_document.uri)?.diagnostics(),
                },
            }),
        ))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        self.info("semantic token partial request").await;
        Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
            result_id: None,
            data: self
                .get_doc(&params.text_document.uri)?
                .semantic_tokens(Some(params.range)),
        })))
    }
    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: self
                .get_doc(&params.text_document.uri)?
                .semantic_tokens(None),
        })))
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> Result<Option<SemanticTokensFullDeltaResult>> {
        self.info(&format!("{params:#?}")).await;
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(Some(
            self.get_doc(&params.text_document.uri)?
                .formatting(params.options),
        ))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.forms.insert(
            params.text_document.uri,
            Document::new(params.text_document.text),
        );
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.forms.remove(&params.text_document.uri);
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let Some(i) = params
            .content_changes
            .iter()
            .enumerate()
            .find_map(|(i, changes)| changes.range.is_none().then_some(i))
        else {
            self.info("client inputted invalid change data").await;
            return;
        };
        let src = params.content_changes.swap_remove(i).text;
        let doc = Document::new(src);
        self.forms.insert(params.text_document.uri, doc);
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let _ = params;
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let _ = params;
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let _ = params;
        self.info("not implemented yet").await;
        Ok(None)
    }
    async fn shutdown(&self) -> Result<()> {
        self.info("cbnf-ls shutdown").await;
        Ok(())
    }
}
