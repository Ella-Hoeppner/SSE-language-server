use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
  client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult::default())
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
}

fn annotate_content_length<'a>(message: &'a str) -> String {
  let content_length = message.len();
  format!("Content-Length: {}\r\n\r\n{}", content_length, message)
}

async fn start_server_terminal() {
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();

  let (service, socket) = LspService::new(|client| Backend { client });

  Server::new(stdin, stdout, socket).serve(service).await;
}
async fn start_server_with_message(message: &'static str) -> String {
  let (mut stdin_writer, stdin_reader) = tokio::io::duplex(1024);
  let (stdout_writer, stdout_reader) = tokio::io::duplex(1024);
  let output = Arc::new(Mutex::new(String::new()));

  // Spawn a task to write the constant string to stdin_writer
  tokio::spawn(async move {
    stdin_writer
      .write_all(annotate_content_length(message).as_bytes())
      .await
      .unwrap();
  });

  // Spawn a task to read from the stdout_reader
  let output_clone = Arc::clone(&output);
  let output_thread = tokio::spawn(async move {
    let mut reader = BufReader::new(stdout_reader);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).await.unwrap();
    let mut output_lock = output_clone.lock().unwrap();
    *output_lock = buffer;
  });

  let (service, socket) = LspService::new(|client| Backend { client });

  Server::new(stdin_reader, stdout_writer, socket)
    .serve(service)
    .await;

  output_thread.await.unwrap();

  // Return the collected output
  let output = Arc::try_unwrap(output)
    .expect("Failed to unwrap Arc")
    .into_inner();
  output.unwrap()
}

#[cfg(test)]
mod tests {
  use crate::{annotate_content_length, start_server_with_message};

  #[tokio::test]
  async fn initialize() {
    let out = start_server_with_message(
      r#"{
        "jsonrpc":"2.0",
        "method":"initialize",
        "params":{
            "processId": null,
            "rootUri": null,
            "capabilities": {}
        },
        "id":1
      }"#,
    )
    .await;
    assert_eq!(
      out,
      annotate_content_length(
        r#"{"jsonrpc":"2.0","result":{"capabilities":{}},"id":1}"#
      )
    )
  }
}
