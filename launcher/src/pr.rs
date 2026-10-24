use crate::versions::{install_dir, pr_dir};
use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::github::{API, EXECUTABLE, REPO, client};

const WORKFLOW: &str = "build.yaml";

#[cfg(target_os = "windows")]
const CI_ARTIFACT: &str = "windows-x64";

#[cfg(target_os = "linux")]
const CI_ARTIFACT: &str = "linux-x64";

#[cfg(target_os = "macos")]
const CI_ARTIFACT: &str = "macos-arm64";

#[derive(Debug, Deserialize)]
struct PullRequestResponse {
    head: PullRequestHead,
}

#[derive(Debug, Deserialize)]
struct PullRequestHead {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct WorkflowRun {
    id: u64,
    status: String,
    conclusion: Option<String>,
    #[serde(default)]
    pull_requests: Vec<WorkflowRunPullRequest>,
}

#[derive(Debug, Deserialize)]
struct WorkflowRunPullRequest {
    number: u64,
    head: PullRequestHead,
}

#[derive(Debug, Deserialize)]
struct WorkflowRunsResponse {
    workflow_runs: Vec<WorkflowRun>,
}

pub fn can_download() -> Result<bool> {
    let status = Command::new("gh").arg("auth").arg("status").output();

    match status {
        Ok(status) if status.status.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(_) => Ok(false),
    }
}

/// Find the latest successful build.yaml run for the current head commit
/// of a pull request.
///
/// Returns the current PR head SHA and the matching workflow run ID.
fn latest_pr_artifact(pr_number: u64) -> Result<(String, u64)> {
    let http = client()?;

    let pr: PullRequestResponse = http
        .get(format!("{API}/repos/{REPO}/pulls/{pr_number}"))
        .send()?
        .error_for_status()?
        .json()
        .context("Failed to fetch pull request")?;

    let head_sha = pr.head.sha;

    // Runs for pull_request events may use a synthetic merge SHA as their
    // head_sha. Match the PR's actual head SHA in pull_requests metadata.
    for page in 1..=10 {
        let url = format!(
            "{API}/repos/{REPO}/actions/workflows/{WORKFLOW}/runs\
             ?event=pull_request&per_page=100&page={page}"
        );

        let runs: WorkflowRunsResponse = http
            .get(url)
            .send()?
            .error_for_status()?
            .json()
            .context("Failed to list workflow runs")?;

        if runs.workflow_runs.is_empty() {
            break;
        }

        for run in &runs.workflow_runs {
            if run.status != "completed" || run.conclusion.as_deref() != Some("success") {
                continue;
            }

            let matches_pr_head = run.pull_requests.iter().any(|associated_pr| {
                associated_pr.number == pr_number && associated_pr.head.sha == head_sha
            });

            if matches_pr_head {
                return Ok((head_sha, run.id));
            }
        }
    }

    bail!(
        "No successful `{WORKFLOW}` build found for PR #{pr_number} \
         at head commit {}. The build may still be running or may have failed.",
        &head_sha[..head_sha.len().min(12)]
    )
}

/// Recursively locate the platform executable downloaded by `gh`.
fn find_executable(directory: &std::path::Path) -> Result<Option<PathBuf>> {
    for entry in fs_err::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.file_name().and_then(|name| name.to_str()) == Some(EXECUTABLE) {
            return Ok(Some(path));
        }

        if path.is_dir()
            && let Some(found) = find_executable(&path)?
        {
            return Ok(Some(found));
        }
    }

    Ok(None)
}

/// Download and install the latest successful PR build for this platform.
///
/// Uses the GitHub CLI to download the authenticated Actions artifact.
/// Returns the installed executable path. PR builds are stored separately
/// from release versions, under a directory keyed by PR number and commit.
pub fn download_pr_build(pr_number: u64) -> Result<PathBuf> {
    let (head_sha, run_id) = latest_pr_artifact(pr_number)?;

    let short_sha = &head_sha[..head_sha.len().min(12)];

    let target_dir = pr_dir(pr_number);
    let target_exe = target_dir.join(EXECUTABLE);

    // Reuse an already installed build for this exact commit.
    if target_exe.is_file() {
        return Ok(target_exe);
    }

    fs_err::create_dir_all(&target_dir)?;

    let temp_dir = install_dir().join(format!(".{short_sha}.staging"));

    let _ = fs_err::remove_dir_all(&temp_dir);
    fs_err::create_dir_all(&temp_dir)?;

    eprintln!("Downloading HEMTT PR #{pr_number} ({short_sha}, run {run_id})...");

    let output = Command::new("gh")
        .args([
            "run",
            "download",
            &run_id.to_string(),
            "--repo",
            REPO,
            "--name",
            CI_ARTIFACT,
            "--dir",
        ])
        .arg(&temp_dir)
        .output()
        .context(
            "Failed to execute `gh`. Install GitHub CLI and authenticate with `gh auth login` \
             to download PR build artifacts.",
        )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let _ = fs_err::remove_dir_all(&temp_dir);

        let details = if !stderr.trim().is_empty() {
            stderr.trim()
        } else {
            stdout.trim()
        };

        bail!(
            "`gh run download` failed for PR #{pr_number}, run {run_id}. \
             Check `gh auth status` and verify that you can access the repository's Actions artifacts.\n\
             {details}"
        );
    }

    let downloaded_exe = match find_executable(&temp_dir)? {
        Some(path) => path,
        None => {
            let _ = fs_err::remove_dir_all(&temp_dir);
            bail!(
                "PR artifact `{CI_ARTIFACT}` did not contain the expected executable \
                 `{EXECUTABLE}`"
            );
        }
    };

    // Normalize the executable location so the installed path is predictable.
    if downloaded_exe != temp_dir.join(EXECUTABLE)
        && let Err(error) = fs_err::copy(&downloaded_exe, temp_dir.join(EXECUTABLE))
    {
        let _ = fs_err::remove_dir_all(&temp_dir);
        return Err(error).context("Failed to stage downloaded HEMTT executable");
    }

    // Publish the completed download atomically within the same filesystem.
    if target_dir.exists() {
        fs_err::remove_dir_all(&target_dir)?;
        fs_err::create_dir_all(&target_dir)?;
    }

    if let Err(error) = fs_err::rename(&temp_dir, &target_dir) {
        let _ = fs_err::remove_dir_all(&temp_dir);
        return Err(error).context("Failed to install PR build");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs_err::metadata(&target_exe)?.permissions();
        permissions.set_mode(0o755);
        fs_err::set_permissions(&target_exe, permissions)?;
    }

    eprintln!("Installed HEMTT PR #{pr_number} ({short_sha}).");

    Ok(target_exe)
}
