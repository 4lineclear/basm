use std::sync::Arc;

use basm::{
    lex::Advance,
    parse::{ParseError, Parser},
    transfer_basm, Basm,
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

fn line_range(line: u32, from: u32, to: u32) -> Range {
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
    source: Arc<str>,
    basm: Basm<Arc<str>>,
    lex: Vec<Advance>,
    errors: Vec<ParseError>,
}

impl Document {
    fn new(source: String) -> Self {
        let source = Arc::<str>::from(source);
        let (basm, errors, lex) = Parser::recorded(&source).parse();
        let basm = transfer_basm(basm, source.clone());
        Self {
            source,
            basm,
            lex,
            errors,
        }
    }
    // #[allow(unused)]
    // fn lit_iter(&self) -> impl Iterator<Item = (u32, &Span, &Literal)> {
    //     self.parser.lines.iter().enumerate().flat_map(|(line, al)| {
    //         al.line
    //             .slice_lit(&self.parser.literals)
    //             .iter()
    //             .map(move |(s, le)| (line as u32, s, le))
    //     })
    // }
    // fn err_iter(&self) -> impl Iterator<Item = (u32, &Span, &LineError)> {
    //     self.lex.lines.iter().enumerate().flat_map(|(line, al)| {
    //         al.line
    //             .slice_err(&self.lex.errors)
    //             .iter()
    //             .map(move |(s, le)| (line as u32, s, le))
    //     })
    // }
    // TODO: add partial & delta semantic token changes
    fn diagnostics(&self) -> Vec<Diagnostic> {
        // use basm::parse::ParseErrorKind::*;
        self.errors
            .iter()
            .map(|e| {
                let ad = e.advance();
                let line = ad.line;
                let span = ad.span;
                let message = e.to_string();
                let range = line_range(line, span.from - ad.offset, span.to - ad.offset);
                Diagnostic {
                    range,
                    message,
                    ..Default::default()
                }
            })
            .collect()
        // fn diagnostic((line, span, err): (u32, &Span, &LineError)) -> Option<Diagnostic> {
        //     let message = match err {
        //         MissingComma => "comma missing".to_owned(),
        //         // UnknownChar(ch) => format!("unexpected char: '{ch}'"),
        //         UnclosedDeref => "Unclosed Deref".to_owned(),
        //         EmptyDeref => "Empty Deref".to_owned(),
        //         MuddyDeref => "Deref Has Other Items Within Range".to_owned(),
        //         // Tab => return None,
        //     };
        //     Some(Diagnostic {
        //         range: line_range(line, span.from, span.to),
        //         message,
        //         ..Default::default()
        //     })
        // }
        //
        // self.err_iter().filter_map(diagnostic).collect()
        // Vec::new()
    }
    // TODO: add partial formatting
    fn formatting(&self, opts: FormattingOptions) -> Vec<TextEdit> {
        let _fmt = FmtContext {
            tab_size: opts.tab_size,
        };
        vec![]
        // basm_fmt::fmt(&self.parser, &fmt)
        //     .map(|e| TextEdit {
        //         range: line_range(e.line, e.span.from, e.span.to),
        //         new_text: e.change,
        //     })
        //     .collect()
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
