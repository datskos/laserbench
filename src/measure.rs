use crate::subscriber::sub_blocks;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc::UnboundedReceiver as Receiver;
use tokio::sync::mpsc::{self, Receiver as BReceiver};
use yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeUpdateBlock};

#[derive(Debug)]
struct LatencyRecord {
    slot: u64,
    time: u64,
}

pub async fn sub_blocks_and_process(url: String, token: String, latency_file: PathBuf) {
    let (tx, rx) = mpsc::unbounded_channel();
    let (subscription_task, processing_task) = tokio::join!(
        tokio::spawn(async move {
            sub_blocks(url, token, tx, CommitmentLevel::Confirmed)
                .await
                .unwrap();
        }),
        tokio::spawn(async move {
            process(rx, latency_file).await.unwrap();
        })
    );

    subscription_task.unwrap();
    processing_task.unwrap();
}

async fn run_latency_writer(mut rx: BReceiver<LatencyRecord>, path: PathBuf) -> eyre::Result<()> {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    let mut writer = BufWriter::new(file);

    if writer.get_ref().metadata().await?.len() == 0 {
        writer.write_all(b"block,time\n").await?;
        writer.flush().await?;
    }

    while let Some(record) = rx.recv().await {
        if let Err(e) = writer
            .write_all(format!("{},{}\n", record.slot, record.time).as_bytes())
            .await
        {
            eprintln!("Failed to write latency to file: {}", e);
            continue;
        }

        if record.slot % 10 == 0 {
            if let Err(e) = writer.flush().await {
                eprintln!("Failed to flush latency file: {}", e);
            }
        }
    }

    Ok(())
}

async fn process(
    mut rx: Receiver<SubscribeUpdateBlock>,
    latency_file: PathBuf,
) -> eyre::Result<()> {
    let (latency_tx, latency_rx) = mpsc::channel(100);
    tokio::spawn(run_latency_writer(latency_rx, latency_file));

    while let Some(block) = rx.recv().await {
        let Some(ts) = block.block_time else {
            continue;
        };
        let Ok(ms) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            continue;
        };

        let slot = block.slot;
        let age_secs = ms.as_secs() as i64 - ts.timestamp;
        let delta = if age_secs < 60 {
            format!("{}s", age_secs)
        } else {
            let minutes = age_secs / 60;
            let seconds = age_secs % 60;
            format!("{}m{}s", minutes, seconds)
        };

        let record = LatencyRecord {
            slot,
            time: ms.as_millis() as u64,
        };
        let _ = latency_tx.send(record).await;

        tracing::info!(?slot, ?delta, "received block");
    }

    Ok(())
}
