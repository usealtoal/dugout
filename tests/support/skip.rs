/// Skip a test if AWS credentials are not configured.
#[macro_export]
macro_rules! skip_without_aws {
    () => {
        if std::env::var("AWS_ACCESS_KEY_ID").is_err() {
            eprintln!("SKIPPED: AWS_ACCESS_KEY_ID not set");
            return;
        }
        if std::env::var("DUGOUT_TEST_KMS_KEY").is_err() {
            eprintln!("SKIPPED: DUGOUT_TEST_KMS_KEY not set (set to an AWS KMS key ARN)");
            return;
        }
    };
}

/// Skip a test if GCP credentials are not configured.
#[macro_export]
macro_rules! skip_without_gcp {
    () => {
        if std::process::Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
            .map(|o| !o.status.success())
            .unwrap_or(true)
        {
            eprintln!("SKIPPED: gcloud not authenticated");
            return;
        }
        if std::env::var("DUGOUT_TEST_GCP_KEY").is_err() {
            eprintln!("SKIPPED: DUGOUT_TEST_GCP_KEY not set (set to a GCP KMS resource name)");
            return;
        }
    };
}
