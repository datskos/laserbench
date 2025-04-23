use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender as Sender;
use yellowstone_grpc_client::GeyserGrpcBuilder;
use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterBlocks, SubscribeRequestPing,
    SubscribeUpdateBlock,
};
use yellowstone_grpc_proto::tonic::transport::ClientTlsConfig;

pub async fn sub_blocks(
    url: String,
    token: String,
    tx: Sender<SubscribeUpdateBlock>,
    commitment: CommitmentLevel,
) -> eyre::Result<()> {
    tracing::debug!("Connecting to {}", url);
    let mut client_builder = GeyserGrpcBuilder::from_shared(url.clone())?
        .timeout(Duration::from_secs(10))
        .max_decoding_message_size(64 * 1024 * 1024)
        .max_encoding_message_size(64 * 1024 * 1024)
        .x_token(Some(token))?;
    if url.starts_with("https") {
        tracing::info!("enabling TLS for gRPC");
        client_builder = client_builder.tls_config(ClientTlsConfig::new().with_native_roots())?;
    }

    let mut client = client_builder.connect().await?;

    let mut blocks = HashMap::new();
    blocks.insert(
        "client".to_string(),
        SubscribeRequestFilterBlocks {
            account_include: vec![],
            include_transactions: Some(true),
            include_accounts: None,
            include_entries: None,
        },
    );

    let req = SubscribeRequest {
        blocks,
        commitment: Some(commitment as i32),
        ..Default::default()
    };
    let (mut sub, mut stream) = client.subscribe_with_request(Some(req.clone())).await?;
    tracing::info!("Connected to endpoint");
    while let Some(message) = stream.next().await {
        tracing::debug!(?message, "got message");
        match message {
            Ok(msg) => match msg.update_oneof {
                Some(UpdateOneof::Ping(_)) => {
                    sub.send(SubscribeRequest {
                        ping: Some(SubscribeRequestPing { id: 1 }),
                        ..Default::default()
                    })
                    .await?;
                }
                Some(UpdateOneof::Block(block)) => {
                    tx.send(block.into())?;
                }
                _ => {}
            },
            Err(err) => {
                tracing::error!(?err, "error from stream. reconnecting");
                tokio::time::sleep(Duration::from_secs(2)).await;
                (sub, stream) = client.subscribe_with_request(Some(req.clone())).await?;
            }
        }
    }

    Ok(())
}
