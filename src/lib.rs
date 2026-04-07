pub mod artifact;
pub mod transcribe;
pub mod cli;
pub mod downloader;
pub mod ffmpeg;
pub mod twitch;
pub mod error;

pub const VALID_FILTER_STAGES: &[&str] = &["queued", "downloaded", "suspect", "failed", "ready"];

pub fn is_valid_filter_stage(s: &str) -> bool {
    VALID_FILTER_STAGES.contains(&s)
}

#[cfg(test)]
mod tests {
    use crate::artifact::ProcessStatus;

    /// Test filter predicate logic for transcribe_all with --force-suspect flag.
    /// Verifies that:
    /// - Normal pending items (downloaded=true, transcribed=false, no suspect outcome) are always included
    /// - Suspect items (transcription_outcome="suspect") are excluded when force_suspect=false
    /// - Suspect items are included when force_suspect=true
    /// - Completed items (transcribed=true) are never included
    #[test]
    fn test_force_suspect_filter_predicate() {
        // Mock status entries
        let normal_pending = ProcessStatus {
            schema_version: 1,
            video_id: "vid_normal".to_string(),
            source_url: "http://example.com/normal".to_string(),
            media_file: Some("audio.m4a".to_string()),
            transcript_file: None,
            downloaded: true,
            transcribed: false,
            last_error: None,
            updated_at_epoch_s: 0,
            transcription_outcome: None,
            transcription_reason: None,
            transcript_word_count: None,
            ready_for_notes: false,
        };

        let suspect_item = ProcessStatus {
            schema_version: 1,
            video_id: "vid_suspect".to_string(),
            source_url: "http://example.com/suspect".to_string(),
            media_file: Some("audio.m4a".to_string()),
            transcript_file: Some("transcript.srt".to_string()),
            downloaded: true,
            transcribed: false,
            last_error: None,
            updated_at_epoch_s: 0,
            transcription_outcome: Some("suspect".to_string()),
            transcription_reason: Some("low_confidence".to_string()),
            transcript_word_count: Some(100),
            ready_for_notes: false,
        };

        let completed_item = ProcessStatus {
            schema_version: 1,
            video_id: "vid_completed".to_string(),
            source_url: "http://example.com/completed".to_string(),
            media_file: Some("audio.m4a".to_string()),
            transcript_file: Some("transcript.srt".to_string()),
            downloaded: true,
            transcribed: true,
            last_error: None,
            updated_at_epoch_s: 0,
            transcription_outcome: Some("completed".to_string()),
            transcription_reason: None,
            transcript_word_count: Some(500),
            ready_for_notes: true,
        };

        // Test with force_suspect=false
        {
            let force_suspect = false;
            let mut included = vec![];

            for (vid, s) in [
                ("normal", &normal_pending),
                ("suspect", &suspect_item),
                ("completed", &completed_item),
            ] {
                let is_suspect = s.transcription_outcome.as_deref() == Some("suspect");
                let include = s.downloaded && ((!s.transcribed && !is_suspect) || (force_suspect && is_suspect));
                if include {
                    included.push(vid);
                }
            }

            // Should only include normal pending
            assert_eq!(included, vec!["normal"]);
        }

        // Test with force_suspect=true
        {
            let force_suspect = true;
            let mut included = vec![];

            for (vid, s) in [
                ("normal", &normal_pending),
                ("suspect", &suspect_item),
                ("completed", &completed_item),
            ] {
                let is_suspect = s.transcription_outcome.as_deref() == Some("suspect");
                let include = s.downloaded && ((!s.transcribed && !is_suspect) || (force_suspect && is_suspect));
                if include {
                    included.push(vid);
                }
            }

            // Should include both normal pending and suspect item
            assert_eq!(included, vec!["normal", "suspect"]);
        }
    }
}
