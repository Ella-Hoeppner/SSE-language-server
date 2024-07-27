use std::{collections::HashMap, sync::RwLock};

use tower_lsp::{
  jsonrpc::Result,
  lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, Hover, HoverContents, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult,
    InitializedParams, MarkedString, MessageType, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
  },
  Client, LanguageServer,
};

#[derive(Debug)]
pub struct Backend {
  pub client: Client,
  documents: RwLock<HashMap<String, String>>,
}

impl Backend {
  pub fn new(client: Client) -> Self {
    Self {
      client,
      documents: Default::default(),
    }
  }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
          TextDocumentSyncKind::FULL,
        )),
        ..Default::default()
      },
      ..Default::default()
    })
  }

  async fn initialized(&self, _: InitializedParams) {
    self
      .client
      .log_message(MessageType::INFO, "server initialized!")
      .await;
  }

  async fn shutdown(&self) -> Result<()> {
    Ok(())
  }

  async fn did_open(&self, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri.to_string();
    let text = params.text_document.text;
    match self.documents.write() {
      Ok(mut docs) => docs.insert(uri, text),
      Err(e) => panic!("did_open failed: {e:?}"),
    };
  }

  async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri.to_string();
    let changes = params.content_changes;
    if let Some(change) = changes.get(0) {
      match self.documents.write() {
        Ok(mut docs) => docs.insert(uri, change.text.clone()),
        Err(e) => panic!("did_change failed: {e:?}"),
      };
    }
  }

  async fn did_close(&self, params: DidCloseTextDocumentParams) {
    let uri = params.text_document.uri.to_string();
    match self.documents.write() {
      Ok(mut docs) => docs.remove(&uri),
      Err(e) => panic!("did_close failed: {e:?}"),
    };
  }

  async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    match self.documents.read() {
      Ok(docs) => {
        let pos_params = params.text_document_position_params;
        let uri = pos_params.text_document.uri.to_string();
        let line = pos_params.position.line;
        let char = pos_params.position.character;
        Ok(docs.get(&uri).map(|doc| Hover {
          contents: HoverContents::Scalar(MarkedString::String(
            format!("{line}:{char}   ") + doc,
          )),
          range: None,
        }))
      }
      Err(e) => panic!("hover failed to read document: {e:?}"),
    }
  }
}
