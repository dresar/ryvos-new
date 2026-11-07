use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use futures::StreamExt;
use ryvos_core::error::RyvosError;

#[derive(Default)]
pub struct SseParser {
    raw_buffer: Vec<u8>,
    buffer: String,
}

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event_type: Option<String>,
    pub data: String,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, chunk: &str) -> Vec<SseEvent> {
        self.feed_bytes(chunk.as_bytes())
    }

    pub fn feed_bytes(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        self.raw_buffer.extend_from_slice(chunk);

        loop {
            if self.raw_buffer.is_empty() {
                break;
            }
            match std::str::from_utf8(&self.raw_buffer) {
                Ok(valid_str) => {
                    self.buffer.push_str(valid_str);
                    self.raw_buffer.clear();
                    break;
                }
                Err(err) => {
                    let valid_up_to = err.valid_up_to();
                    if valid_up_to > 0 {
                        let valid_str = std::str::from_utf8(&self.raw_buffer[..valid_up_to]).unwrap();
                        self.buffer.push_str(valid_str);
                        self.raw_buffer.drain(..valid_up_to);
                    }
                    if let Some(error_len) = err.error_len() {
                        self.raw_buffer.drain(..error_len);
                    } else {
                        break;
                    }
                }
            }
        }

        self.extract_events()
    }

    fn extract_events(&mut self) -> Vec<SseEvent> {
        let mut events = Vec::new();

        while let Some((pos, delim_len)) = find_delimiter(&self.buffer) {
            let block: String = self.buffer[..pos].to_string();
            self.buffer.drain(..pos + delim_len);

            let mut event_type = None;
            let mut data_lines = Vec::new();

            let normalized = block.replace("\r\n", "\n").replace('\r', "\n");
            for line in normalized.lines() {
                if let Some(val) = line.strip_prefix("event: ") {
                    event_type = Some(val.to_string());
                } else if let Some(val) = line.strip_prefix("event:") {
                    event_type = Some(val.trim_start().to_string());
                } else if let Some(val) = line.strip_prefix("data: ") {
                    data_lines.push(val.to_string());
                } else if let Some(val) = line.strip_prefix("data:") {
                    data_lines.push(val.to_string());
                }
            }

            if !data_lines.is_empty() {
                events.push(SseEvent {
                    event_type,
                    data: data_lines.join("\n"),
                });
            }
        }

        events
    }
}

fn find_delimiter(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if i + 4 <= len && &bytes[i..i + 4] == b"\r\n\r\n" {
            return Some((i, 4));
        }
        if i + 3 <= len && &bytes[i..i + 3] == b"\r\n\n" {
            return Some((i, 3));
        }
        if i + 3 <= len && &bytes[i..i + 3] == b"\n\r\n" {
            return Some((i, 3));
        }
        if i + 2 <= len && &bytes[i..i + 2] == b"\n\n" {
            return Some((i, 2));
        }
        if i + 2 <= len && &bytes[i..i + 2] == b"\r\r" {
            return Some((i, 2));
        }
        i += 1;
    }
    None
}

pub struct SseStream<S> {
    inner: S,
    parser: SseParser,
    pending: Vec<SseEvent>,
    idle_timeout: Duration,
    timer: Option<Pin<Box<tokio::time::Sleep>>>,
}

impl<S> SseStream<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            parser: SseParser::new(),
            pending: Vec::new(),
            idle_timeout: Duration::from_secs(15),
            timer: None,
        }
    }

    pub fn with_idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    pub async fn next_chunk_timeout(
        &mut self,
    ) -> Result<Option<bytes::Bytes>, RyvosError>
    where
        S: Stream<Item = std::result::Result<bytes::Bytes, reqwest::Error>> + Unpin,
    {
        match tokio::time::timeout(self.idle_timeout, self.inner.next()).await {
            Ok(Some(Ok(bytes))) => Ok(Some(bytes)),
            Ok(Some(Err(e))) => Err(RyvosError::LlmStream(e.to_string())),
            Ok(None) => Ok(None),
            Err(_) => Err(RyvosError::LlmStream("Per-chunk idle timeout (15s) expired".into())),
        }
    }
}

impl<S> Stream for SseStream<S>
where
    S: Stream<Item = std::result::Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    type Item = SseEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if !this.pending.is_empty() {
            this.timer = None;
            return Poll::Ready(Some(this.pending.remove(0)));
        }

        match Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                this.timer = None;
                let mut events = this.parser.feed_bytes(&bytes);
                if events.is_empty() {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    let first = events.remove(0);
                    this.pending = events;
                    Poll::Ready(Some(first))
                }
            }
            Poll::Ready(Some(Err(e))) => {
                this.timer = None;
                tracing::warn!(error = %e, "SSE stream error");
                Poll::Ready(None)
            }
            Poll::Ready(None) => {
                this.timer = None;
                Poll::Ready(None)
            }
            Poll::Pending => {
                let timer = this.timer.get_or_insert_with(|| {
                    Box::pin(tokio::time::sleep(this.idle_timeout))
                });
                if timer.as_mut().poll(cx).is_ready() {
                    tracing::warn!("SSE stream per-chunk idle timeout expired");
                    this.timer = None;
                    Poll::Ready(None)
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_parser_basic() {
        let mut parser = SseParser::new();
        let events = parser.feed("event: message_start\ndata: {\"type\":\"message_start\"}\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type.as_deref(), Some("message_start"));
        assert_eq!(events[0].data, "{\"type\":\"message_start\"}");
    }

    #[test]
    fn test_sse_parser_multiple_events() {
        let mut parser = SseParser::new();
        let events = parser.feed("event: a\ndata: {\"x\":1}\n\nevent: b\ndata: {\"x\":2}\n\n");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_sse_parser_chunked() {
        let mut parser = SseParser::new();
        let events = parser.feed("event: a\ndata: {\"x\":");
        assert_eq!(events.len(), 0);
        let events = parser.feed("1}\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "{\"x\":1}");
    }

    #[test]
    fn test_sse_parser_crlf_and_cr() {
        let mut parser = SseParser::new();
        let events = parser.feed("event: a\r\ndata: 1\r\n\r\nevent: b\rdata: 2\r\r");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "1");
        assert_eq!(events[1].data, "2");
    }

    #[test]
    fn test_sse_parser_split_utf8() {
        let mut parser = SseParser::new();
        let text = "data: 🦀\n\n";
        let bytes = text.as_bytes();
        let ev1 = parser.feed_bytes(&bytes[..7]);
        assert!(ev1.is_empty());
        let ev2 = parser.feed_bytes(&bytes[7..]);
        assert_eq!(ev2.len(), 1);
        assert_eq!(ev2[0].data, "🦀");
    }

    #[test]
    fn test_sse_parser_invalid_utf8_recovery() {
        let mut parser = SseParser::new();
        let mut bad_chunk = Vec::new();
        bad_chunk.extend_from_slice(b"data: hello");
        bad_chunk.push(0xFF);
        bad_chunk.push(0xFE);
        bad_chunk.extend_from_slice(b"world\n\n");
        let ev = parser.feed_bytes(&bad_chunk);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].data, "helloworld");
    }
}
