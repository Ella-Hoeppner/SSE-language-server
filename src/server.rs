use tower_lsp::{
  jsonrpc::Result,
  lsp_types::{
    Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, MarkedString,
    MessageType, ServerCapabilities,
  },
  Client, LanguageServer,
};

#[derive(Debug)]
pub struct Backend {
  pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
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

  async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
    Ok(Some(Hover {
      contents: HoverContents::Scalar(MarkedString::String(
        "hovering!!".to_string(),
      )),
      range: None,
    }))
  }
}
