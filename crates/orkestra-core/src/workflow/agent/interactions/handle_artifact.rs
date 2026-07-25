//! Handle artifact output: store artifact, check gate, auto-approve or await review.

use crate::workflow::config::WorkflowConfig;
use crate::workflow::domain::Task;
use crate::workflow::iteration::IterationService;
use crate::workflow::ports::WorkflowResult;
use crate::workflow::runtime::{Artifact, TaskState};
use crate::workflow::stage::interactions as stage;

/// Store an artifact and transition state based on the pre-resolved gate decision.
///
/// `has_automated_gate` must be resolved by the caller (see `dispatch_output`) to
/// avoid duplicating gate logic and to keep this function free of filesystem I/O.
pub fn execute(
    workflow: &WorkflowConfig,
    iteration_service: &IterationService,
    task: &mut Task,
    content: &str,
    stage_name: &str,
    now: &str,
    has_automated_gate: bool,
) -> WorkflowResult<Option<String>> {
    let artifact_name = stage::finalize_advancement::artifact_name_for_stage(
        workflow, &task.flow, stage_name, "artifact",
    );
    task.artifacts
        .set(Artifact::new(&artifact_name, content, stage_name, now));

    if has_automated_gate {
        task.state = TaskState::awaiting_gate(stage_name);
        task.updated_at = now.to_string();
        return Ok(Some(artifact_name));
    }

    stage::auto_advance_or_review::execute(iteration_service, workflow, task, stage_name, now)?;
    Ok(Some(artifact_name))
}
