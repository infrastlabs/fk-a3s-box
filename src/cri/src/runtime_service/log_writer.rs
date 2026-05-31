//! CRI container log writer.
//!
//! Buffers workload stdout/stderr into line-oriented CRI log records
//! (`<timestamp> <stream> F <line>`) for [`super::supervisor`].

use tokio::io::AsyncWriteExt;

pub(super) struct CriLogWriter {
    file: tokio::fs::File,
    path: String,
    stdout_partial: Vec<u8>,
    stderr_partial: Vec<u8>,
}

impl CriLogWriter {
    pub(super) async fn open(log_path: &str) -> std::io::Result<Option<Self>> {
        if log_path.is_empty() {
            return Ok(None);
        }

        let path = std::path::Path::new(log_path);
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            tokio::fs::create_dir_all(parent).await?;
        }

        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;

        Ok(Some(Self {
            file,
            path: log_path.to_string(),
            stdout_partial: Vec::new(),
            stderr_partial: Vec::new(),
        }))
    }

    /// Reopen the log file at its path (CRI `ReopenContainerLog`). The kubelet
    /// rotates by renaming the current file, then calls this; we flush, drop the
    /// old handle, and open a fresh file at the original path so subsequent
    /// output lands where the kubelet now expects it.
    pub(super) async fn reopen(&mut self) -> std::io::Result<()> {
        self.flush_partials().await?;
        if let Some(parent) = std::path::Path::new(&self.path)
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            tokio::fs::create_dir_all(parent).await?;
        }
        self.file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        Ok(())
    }

    pub(super) async fn write_chunk(
        &mut self,
        stream: a3s_box_core::exec::StreamType,
        data: &[u8],
    ) -> std::io::Result<()> {
        let partial = match stream {
            a3s_box_core::exec::StreamType::Stdout => &mut self.stdout_partial,
            a3s_box_core::exec::StreamType::Stderr => &mut self.stderr_partial,
        };

        partial.extend_from_slice(data);
        let mut complete_lines = Vec::new();
        while let Some(newline) = partial.iter().position(|byte| *byte == b'\n') {
            let mut line: Vec<u8> = partial.drain(..=newline).collect();
            line.pop();
            complete_lines.push(line);
        }

        for line in complete_lines {
            self.write_record(stream, &line).await?;
        }

        Ok(())
    }

    pub(super) async fn flush_partials(&mut self) -> std::io::Result<()> {
        if !self.stdout_partial.is_empty() {
            let line = std::mem::take(&mut self.stdout_partial);
            self.write_record(a3s_box_core::exec::StreamType::Stdout, &line)
                .await?;
        }
        if !self.stderr_partial.is_empty() {
            let line = std::mem::take(&mut self.stderr_partial);
            self.write_record(a3s_box_core::exec::StreamType::Stderr, &line)
                .await?;
        }

        self.file.flush().await
    }

    async fn write_record(
        &mut self,
        stream: a3s_box_core::exec::StreamType,
        line: &[u8],
    ) -> std::io::Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
        let stream = match stream {
            a3s_box_core::exec::StreamType::Stdout => "stdout",
            a3s_box_core::exec::StreamType::Stderr => "stderr",
        };

        self.file.write_all(timestamp.as_bytes()).await?;
        self.file.write_all(b" ").await?;
        self.file.write_all(stream.as_bytes()).await?;
        self.file.write_all(b" F ").await?;
        self.file.write_all(line).await?;
        self.file.write_all(b"\n").await
    }
}
