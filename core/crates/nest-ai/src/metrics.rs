//! Completion timing and token usage metrics.

/// Token counts and throughput for a completed inference request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompletionMetrics {
    /// Tokens consumed evaluating the prompt.
    pub prompt_tokens: u32,
    /// Tokens generated in the response.
    pub completion_tokens: u32,
    /// Sum of prompt and completion tokens.
    pub total_tokens: u32,

    /// Prompt evaluation throughput (tokens / second).
    pub prompt_tps: f64,
    /// Generation throughput (tokens / second).
    pub generation_tps: f64,
    /// End-to-end throughput (total tokens / second).
    pub overall_tps: f64,

    /// Model load time in milliseconds.
    pub load_time_ms: u64,
    /// Prompt evaluation time in milliseconds.
    pub prompt_time_ms: u64,
    /// Generation time in milliseconds.
    pub generation_time_ms: u64,
    /// Total request time in milliseconds.
    pub total_time_ms: u64,
}

impl CompletionMetrics {
    /// Builds metrics from provider timing fields (durations in nanoseconds).
    pub fn from_timing(
        prompt_tokens: u32,
        completion_tokens: u32,
        load_duration_ns: u64,
        prompt_eval_duration_ns: u64,
        eval_duration_ns: u64,
        total_duration_ns: u64,
    ) -> Self {
        let total_tokens = prompt_tokens.saturating_add(completion_tokens);
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens,
            prompt_tps: tokens_per_second(prompt_tokens, prompt_eval_duration_ns),
            generation_tps: tokens_per_second(completion_tokens, eval_duration_ns),
            overall_tps: tokens_per_second(total_tokens, total_duration_ns),
            load_time_ms: ns_to_ms(load_duration_ns),
            prompt_time_ms: ns_to_ms(prompt_eval_duration_ns),
            generation_time_ms: ns_to_ms(eval_duration_ns),
            total_time_ms: ns_to_ms(total_duration_ns),
        }
    }

    /// Compact label for status bars (`135 tok · 21.6 tok/s · 8.9s`).
    pub fn status_label(&self) -> String {
        format!(
            "{} tok · {:.1} tok/s · {:.1}s",
            self.total_tokens,
            self.generation_tps,
            self.total_time_ms as f64 / 1000.0
        )
    }

    /// Detailed label for inline chat stats.
    pub fn detail_label(&self) -> String {
        format!(
            "Prompt: {} tok · {:.1} tok/s · {:.1}s  ·  Output: {} tok · {:.1} tok/s · {:.1}s",
            self.prompt_tokens,
            self.prompt_tps,
            self.prompt_time_ms as f64 / 1000.0,
            self.completion_tokens,
            self.generation_tps,
            self.generation_time_ms as f64 / 1000.0,
        )
    }
}

fn ns_to_ms(ns: u64) -> u64 {
    ns / 1_000_000
}

fn tokens_per_second(tokens: u32, duration_ns: u64) -> f64 {
    if tokens == 0 || duration_ns == 0 {
        return 0.0;
    }
    tokens as f64 / (duration_ns as f64 / 1_000_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_timing_matches_ollama_sample() {
        let metrics = CompletionMetrics::from_timing(
            37,
            98,
            0,
            643_843_000,
            4_538_229_000,
            8_908_920_680,
        );

        assert_eq!(metrics.prompt_tokens, 37);
        assert_eq!(metrics.completion_tokens, 98);
        assert_eq!(metrics.total_tokens, 135);
        assert_eq!(metrics.prompt_time_ms, 643);
        assert_eq!(metrics.generation_time_ms, 4_538);
        assert_eq!(metrics.total_time_ms, 8_908);
        assert!((metrics.generation_tps - 21.6).abs() < 0.2);
    }
}
