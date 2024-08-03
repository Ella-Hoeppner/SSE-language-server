use std::{collections::HashMap, sync::RwLock};

use serde_json::Value;
use sse::{
  str_tagged::{StringTaggedDocument, StringTaggedSyntaxGraph},
  Parser,
};
use tower_lsp::{
  jsonrpc::{Error, Result},
  lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, ExecuteCommandOptions, ExecuteCommandParams,
    Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, MarkedString,
    MessageType, ServerCapabilities, TextDocumentPositionParams,
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

fn sexp_graph<'g>() -> StringTaggedSyntaxGraph<'g> {
  StringTaggedSyntaxGraph::contextless_from_descriptions(
    vec![
      ' '.to_string(),
      '\n'.to_string(),
      '\t'.to_string(),
      '\r'.to_string(),
    ],
    Some('\\'.to_string()),
    vec![("", "(", ")")],
    vec![],
  )
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
        execute_command_provider: Some(ExecuteCommandOptions {
          commands: vec![
            "expandSelection".to_string(),
            "moveCursorLeft".to_string(),
          ],
          work_done_progress_options: Default::default(),
        }),
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

  async fn execute_command(
    &self,
    params: ExecuteCommandParams,
  ) -> Result<Option<Value>> {
    match params.command.as_str() {
      "expandSelection" => {
        if params.arguments.len() == 1 {
          if let Some((selection_start_params, selection_end_params)) =
            serde_json::from_value::<(
              TextDocumentPositionParams,
              TextDocumentPositionParams,
            )>(params.arguments[0].clone())
            .ok()
          {
            match self.documents.read() {
              Ok(docs) => {
                let uri = selection_start_params.text_document.uri.to_string();
                let text = docs
                  .get(&uri)
                  .expect(&format!("didn't have data for document {}", uri));
                let document: StringTaggedDocument =
                  Parser::new(sexp_graph(), text).try_into().unwrap();
                let document_start_index = document
                  .row_and_col_to_index(
                    selection_start_params.position.line as usize,
                    selection_start_params.position.character as usize,
                  )
                  .expect("invalid row and col");
                let document_end_index = document
                  .row_and_col_to_index(
                    selection_end_params.position.line as usize,
                    selection_end_params.position.character as usize,
                  )
                  .expect("invalid row and col");
                Ok(
                  document
                    .expand_selection(
                      &(document_start_index..document_end_index),
                    )
                    .map(|new_selection| {
                      let (start_row, start_col) = document
                        .index_to_row_and_col(new_selection.start)
                        .unwrap();
                      let (end_row, end_col) = document
                        .index_to_row_and_col(new_selection.end)
                        .unwrap();
                      serde_json::to_value([
                        start_row, start_col, end_row, end_col,
                      ])
                      .unwrap()
                    }),
                )
              }
              Err(e) => {
                panic!("expandSelection failed to read document: {e:?}")
              }
            }
          } else {
            Err(Error::invalid_params(format!(
              "Invalid parameters: {}",
              params.arguments[0]
            )))
          }
        } else {
          Err(Error::invalid_params(format!(
            "Invalid number of arguments {}",
            params.arguments.len()
          )))
        }
      }
      "moveCursorLeft" => {
        if params.arguments.len() == 1 {
          if let Some((selection_start_params, selection_end_params)) =
            serde_json::from_value::<(
              TextDocumentPositionParams,
              TextDocumentPositionParams,
            )>(params.arguments[0].clone())
            .ok()
          {
            match self.documents.read() {
              Ok(docs) => {
                let uri = selection_start_params.text_document.uri.to_string();
                let text = docs
                  .get(&uri)
                  .expect(&format!("didn't have data for document {}", uri));
                let document: StringTaggedDocument =
                  Parser::new(sexp_graph(), text).try_into().unwrap();
                let document_start_index = document
                  .row_and_col_to_index(
                    selection_start_params.position.line as usize,
                    selection_start_params.position.character as usize,
                  )
                  .expect("invalid row and col");
                let document_end_index = document
                  .row_and_col_to_index(
                    selection_end_params.position.line as usize,
                    selection_end_params.position.character as usize,
                  )
                  .expect("invalid row and col");
                let (start_row, start_col) = document
                  .index_to_row_and_col(
                    document
                      .get_subtree(&document.innermost_enclosing_path(
                        &(document_start_index..document_end_index),
                      ))
                      .unwrap()
                      .range()
                      .start,
                  )
                  .unwrap();
                Ok(Some(serde_json::to_value([start_row, start_col]).unwrap()))
              }
              Err(e) => panic!("moveCursorLeft failed to read document: {e:?}"),
            }
          } else {
            Err(Error::invalid_params(format!(
              "Invalid parameters: {}",
              params.arguments[0]
            )))
          }
        } else {
          Err(Error::invalid_params(format!(
            "Invalid number of arguments {}",
            params.arguments.len()
          )))
        }
      }
      _ => Err(Error::method_not_found()),
    }
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
        Ok(docs.get(&uri).map(|text| {
          let document: StringTaggedDocument =
            Parser::new(sexp_graph(), text).try_into().unwrap();
          let index = document
            .row_and_col_to_index(line as usize, char as usize)
            .unwrap();
          let hovered_path = document.innermost_enclosing_path(&(index..index));
          Hover {
            contents: HoverContents::Scalar(MarkedString::String(format!(
              "{:?}",
              hovered_path
            ))),
            range: None,
          }
        }))
      }
      Err(e) => panic!("hover failed to read document: {e:?}"),
    }
  }
}
