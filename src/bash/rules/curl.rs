//! Rules for `curl` operations.
//!
//! Allows read-only `curl` invocations (default GET, or explicit `-X GET`/`-X HEAD`)
//! that do not write to local files, upload data, send body payloads, or mutate
//! remote state. Any flag that writes to disk, uploads, sends body data, or
//! reads opaque config excludes the command from the allow list.

use crate::prelude::*;

/// Rules for `curl`.
#[must_use]
pub fn curl_rules() -> Vec<BashRule> {
    vec![curl__read_only()]
}

/// Allow `curl` with no write, upload, body, or state-mutating flags.
fn curl__read_only() -> BashRule {
    BashRule {
        id: "curl__read_only".to_owned(),
        command: "curl".to_owned(),
        without_any: Some(vec![
            ArgMatcher::new("-X").ivalue("{POST,PUT,PATCH,DELETE}"),
            ArgMatcher::new("--request").ivalue("{POST,PUT,PATCH,DELETE}"),
            ArgMatcher::new("-d"),
            ArgMatcher::new("--data"),
            ArgMatcher::new("--data-raw"),
            ArgMatcher::new("--data-binary"),
            ArgMatcher::new("--data-urlencode"),
            ArgMatcher::new("--data-ascii"),
            ArgMatcher::new("--json"),
            ArgMatcher::new("-F"),
            ArgMatcher::new("--form"),
            ArgMatcher::new("--form-string"),
            ArgMatcher::new("-T"),
            ArgMatcher::new("--upload-file"),
            ArgMatcher::new("-a"),
            ArgMatcher::new("--append"),
            ArgMatcher::new("-o"),
            ArgMatcher::new("--output"),
            ArgMatcher::new("-O"),
            ArgMatcher::new("--remote-name"),
            ArgMatcher::new("--remote-name-all"),
            ArgMatcher::new("-J"),
            ArgMatcher::new("--remote-header-name"),
            ArgMatcher::new("--output-dir"),
            ArgMatcher::new("--create-dirs"),
            ArgMatcher::new("--create-file-mode"),
            ArgMatcher::new("--no-clobber"),
            ArgMatcher::new("--skip-existing"),
            ArgMatcher::new("--remove-on-error"),
            ArgMatcher::new("-c"),
            ArgMatcher::new("--cookie-jar"),
            ArgMatcher::new("-D"),
            ArgMatcher::new("--dump-header"),
            ArgMatcher::new("--etag-save"),
            ArgMatcher::new("--alt-svc"),
            ArgMatcher::new("--hsts"),
            ArgMatcher::new("--ssl-sessions"),
            ArgMatcher::new("--trace"),
            ArgMatcher::new("--trace-ascii"),
            ArgMatcher::new("--stderr"),
            ArgMatcher::new("--libcurl"),
            ArgMatcher::new("-K"),
            ArgMatcher::new("--config"),
            ArgMatcher::new("--mail-from"),
            ArgMatcher::new("--mail-rcpt"),
            ArgMatcher::new("--mail-auth"),
            ArgMatcher::new("--mail-rcpt-allowfails"),
            ArgMatcher::new("-Q"),
            ArgMatcher::new("--quote"),
            ArgMatcher::new("--ftp-create-dirs"),
        ]),
        outcome: Outcome::allow("Read-only `curl`"),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn curl_bare_get() {
        let outcome = evaluate_expect_outcome("curl https://example.com");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn curl_silent_pipe_jq() {
        let outcome = evaluate_expect_outcome(
            "curl -s https://crates.io/api/v1/crates/serde | jq '.crate.description, .versions[0].features'",
        );
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn curl_silent_pipe_jq_with_stderr_dev_null() {
        let outcome = evaluate_expect_outcome(
            "curl -s https://crates.io/api/v1/crates/serde | jq '.crate.description' 2>/dev/null",
        );
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn curl_explicit_get() {
        let outcome = evaluate_expect_outcome("curl -X GET https://example.com");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn curl_explicit_head() {
        let outcome = evaluate_expect_outcome("curl -X HEAD https://example.com");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn curl_with_header() {
        let outcome = evaluate_expect_outcome(
            "curl -H 'Accept: application/json' https://api.example.com/users",
        );
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn curl_follow_redirects() {
        let outcome = evaluate_expect_outcome("curl -L https://example.com");
        assert_eq!(outcome.decision, Decision::Allow);
    }

    #[test]
    fn curl_post_method() {
        let reason = evaluate_expect_skip("curl -X POST https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_put_method() {
        let reason = evaluate_expect_skip("curl --request PUT https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_delete_method() {
        let reason = evaluate_expect_skip("curl -X DELETE https://example.com/items/1");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_data_flag() {
        let reason = evaluate_expect_skip("curl -d @body.json https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_data_raw() {
        let reason = evaluate_expect_skip("curl --data-raw 'x=1' https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_json_flag() {
        let reason = evaluate_expect_skip("curl --json '{\"x\":1}' https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_form_flag() {
        let reason = evaluate_expect_skip("curl -F file=@a.txt https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_upload_file() {
        let reason = evaluate_expect_skip("curl -T file.txt https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_output_flag() {
        let reason = evaluate_expect_skip("curl -o out.html https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_remote_name() {
        let reason = evaluate_expect_skip("curl -O https://example.com/file.zip");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_output_dir() {
        let reason = evaluate_expect_skip("curl --output-dir ./downloads -O https://example.com/x");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_cookie_jar() {
        let reason = evaluate_expect_skip("curl -c cookies.txt https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_dump_header() {
        let reason = evaluate_expect_skip("curl -D headers.txt https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_config_flag() {
        let reason = evaluate_expect_skip("curl -K my.cfg https://example.com");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_quote_command() {
        let reason = evaluate_expect_skip("curl -Q 'DELE old.txt' ftp://example.com/");
        assert_eq!(reason, SkipReason::NoMatches);
    }

    #[test]
    fn curl_redirect_to_file() {
        let reason = evaluate_expect_skip("curl https://example.com > /tmp/out.html");
        assert_eq!(reason, SkipReason::UnsafeRedirect);
    }
}
