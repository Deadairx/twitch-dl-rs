use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Deprecated: kept for backward compatibility during transition to hear-based transcription
// Will be removed in T02 when main.rs is updated to use transcribe_to_srt_and_vtt
#[deprecated]
#[allow(dead_code)]
pub fn transcribe_to_txt(
    _media_path: &Path,
    _artifact_dir: &Path,
) -> Result<PathBuf, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "transcribe_to_txt is deprecated; use transcribe_to_srt_and_vtt instead",
    ))
}

#[derive(Debug)]
pub enum TranscriptionOutcome {
    Completed {
        srt_path: PathBuf,
        #[allow(dead_code)]
        vtt_path: PathBuf,
        word_count: u64,
    },
    Suspect {
        srt_path: PathBuf,
        #[allow(dead_code)]
        vtt_path: PathBuf,
        word_count: u64,
        reason: String,
    },
    Failed {
        reason: String,
    },
}

/// Convert SRT format to VTT format.
/// Algorithm: prepend WEBVTT header; skip sequence numbers (all-numeric lines);
/// replace ',' with '.' in timestamp lines; copy text and blank lines verbatim.
fn srt_to_vtt(srt: &str) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for line in srt.lines() {
        let trimmed = line.trim();
        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            // Skip sequence numbers
            continue;
        } else if trimmed.contains(" --> ") {
            // Replace comma with period in timestamps
            out.push_str(&line.replace(',', "."));
            out.push('\n');
        } else {
            // Copy text and blank lines verbatim
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Extract word list from SRT text (excluding timestamps and sequence numbers).
fn extract_words_from_srt(srt: &str) -> Vec<String> {
    let mut words = Vec::new();
    for line in srt.lines() {
        let trimmed = line.trim();
        // Skip sequence numbers (all digits), timestamps, and blank lines
        if trimmed.is_empty()
            || trimmed.chars().all(|c| c.is_ascii_digit())
            || trimmed.contains(" --> ")
        {
            continue;
        }
        // Split on whitespace and collect non-empty words
        for word in trimmed.split_whitespace() {
            words.push(word.to_string());
        }
    }
    words
}

/// Check if word count is below threshold based on duration.
/// Threshold: 50 words per hour.
fn check_word_count_threshold(
    word_count: u64,
    duration_secs: f64,
) -> Option<String> {
    let threshold = (duration_secs / 3600.0) * 50.0;
    if (word_count as f64) < threshold {
        Some(format!(
            "word count {} below threshold {:.0} for {:.0}s audio",
            word_count, threshold, duration_secs
        ))
    } else {
        None
    }
}

/// Check for repeated trigrams in a sliding 200-word window.
fn check_repetition_heuristic(words: &[String]) -> Option<String> {
    const WINDOW_SIZE: usize = 200;

    if words.len() < 3 {
        return None;
    }

    // Slide a 200-word window over the word list
    for start in 0..=words.len().saturating_sub(WINDOW_SIZE) {
        let end = (start + WINDOW_SIZE).min(words.len());
        let window = &words[start..end];

        // Build trigrams in this window
        let mut trigram_counts: HashMap<Vec<String>, usize> = HashMap::new();
        for i in 0..window.len().saturating_sub(2) {
            let trigram = vec![
                window[i].clone(),
                window[i + 1].clone(),
                window[i + 2].clone(),
            ];
            *trigram_counts.entry(trigram).or_insert(0) += 1;
        }

        // Check if any trigram appears > 10 times
        for count in trigram_counts.values() {
            if *count > 10 {
                return Some("repeated trigram detected in 200-word window".to_string());
            }
        }
    }

    None
}

/// Transcribe media using hear, capture SRT, convert to VTT, and apply quality heuristics.
pub fn transcribe_to_srt_and_vtt(
    media_path: &Path,
    artifact_dir: &Path,
    duration_secs: Option<f64>,
) -> TranscriptionOutcome {
    let srt_path = artifact_dir.join("transcript.srt");
    let vtt_path = artifact_dir.join("transcript.vtt");

    // Clean up stale files from prior failed runs
    let _ = fs::remove_file(&srt_path);
    let _ = fs::remove_file(&vtt_path);

    // Invoke hear with -d -i <audio-file> -S
    let output = match Command::new("hear")
        .args(["-d", "-i"])
        .arg(media_path)
        .arg("-S")
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            return TranscriptionOutcome::Failed {
                reason: format!("Failed to launch hear: {}", e),
            }
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return TranscriptionOutcome::Failed {
            reason: format!("hear exited with status {}: {}", output.status, stderr),
        };
    }

    let srt_content = String::from_utf8_lossy(&output.stdout);

    // Write SRT file
    if let Err(e) = fs::write(&srt_path, srt_content.as_bytes()) {
        return TranscriptionOutcome::Failed {
            reason: format!("Failed to write SRT file: {}", e),
        };
    }

    // Convert to VTT
    let vtt_content = srt_to_vtt(&srt_content);

    // Write VTT file
    if let Err(e) = fs::write(&vtt_path, vtt_content.as_bytes()) {
        return TranscriptionOutcome::Failed {
            reason: format!("Failed to write VTT file: {}", e),
        };
    }

    // Extract words and count
    let words = extract_words_from_srt(&srt_content);
    let word_count = words.len() as u64;

    // Apply quality heuristics in order
    // 1. Word-count threshold
    if let Some(duration) = duration_secs {
        if let Some(reason) = check_word_count_threshold(word_count, duration) {
            return TranscriptionOutcome::Suspect {
                srt_path,
                vtt_path,
                word_count,
                reason,
            };
        }
    }

    // 2. Repetition detection
    if let Some(reason) = check_repetition_heuristic(&words) {
        return TranscriptionOutcome::Suspect {
            srt_path,
            vtt_path,
            word_count,
            reason,
        };
    }

    // All checks passed
    TranscriptionOutcome::Completed {
        srt_path,
        vtt_path,
        word_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_to_vtt_basic() {
        let srt = "1\n00:00:00,000 --> 00:00:05,000\nHello world\n\n2\n00:00:05,000 --> 00:00:10,000\nSecond line";
        let vtt = srt_to_vtt(srt);
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:00.000 --> 00:00:05.000")); // comma replaced
        assert!(vtt.contains("Hello world"));
        assert!(vtt.contains("Second line"));
        assert!(!vtt.contains("1\n")); // no sequence numbers
        assert!(!vtt.contains("2\n"));
    }

    #[test]
    fn test_srt_to_vtt_resetting_sequence() {
        let srt = "1\n00:00:00,000 --> 00:00:05,000\nText one\n\n1\n00:00:05,000 --> 00:00:10,000\nText two";
        let vtt = srt_to_vtt(srt);
        // Both sequence "1"s should be stripped
        let line_count = vtt.lines().filter(|l| l.trim() == "1").count();
        assert_eq!(line_count, 0);
        assert!(vtt.contains("Text one"));
        assert!(vtt.contains("Text two"));
    }

    #[test]
    fn test_word_count_threshold_flags_suspect() {
        // 7200 seconds = 2 hours → threshold = 100 words
        // word_count = 10 → 10 < 100 → should flag suspect
        let word_count = 10u64;
        let duration_secs = 7200.0;
        let reason = check_word_count_threshold(word_count, duration_secs);
        assert!(reason.is_some());
        let r = reason.unwrap();
        assert!(r.contains("word count 10 below threshold"));
    }

    #[test]
    fn test_word_count_threshold_passes() {
        // 7200 seconds = 2 hours → threshold = 100 words
        // word_count = 150 → 150 >= 100 → should pass
        let word_count = 150u64;
        let duration_secs = 7200.0;
        let reason = check_word_count_threshold(word_count, duration_secs);
        assert!(reason.is_none());
    }

    #[test]
    fn test_repetition_heuristic_flags_suspect() {
        // Build a 200-word list with a trigram repeated 11 times
        let mut words = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        // Repeat trigram 11 times (each repetition adds 3 words)
        for _ in 0..11 {
            words.push("repeat".to_string());
            words.push("trigram".to_string());
            words.push("here".to_string());
        }
        // Fill to 200 words with unique words
        for i in 0..(200 - words.len()) {
            words.push(format!("word{}", i));
        }
        let reason = check_repetition_heuristic(&words);
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("repeated trigram"));
    }

    #[test]
    fn test_repetition_heuristic_clean_input() {
        // Build a 200-word list with no repetition
        let mut words = Vec::new();
        for i in 0..200 {
            words.push(format!("unique_word_{}", i));
        }
        let reason = check_repetition_heuristic(&words);
        assert!(reason.is_none());
    }

    #[test]
    fn test_extract_words_from_srt() {
        let srt = "1\n00:00:00,000 --> 00:00:05,000\nHello world test\n\n2\n00:00:05,000 --> 00:00:10,000\nSecond line here";
        let words = extract_words_from_srt(srt);
        assert_eq!(words.len(), 6); // "Hello", "world", "test", "Second", "line", "here"
        assert_eq!(words[0], "Hello");
        assert_eq!(words[1], "world");
    }
}
