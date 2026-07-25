//! Handle artifact output: store artifact, check gate, auto-approve or await review.

use std::path::Path;

use crate::workflow::config::{load_and_resolve_techniques, GateConfig, WorkflowConfig};
use crate::workflow::domain::Task;
use crate::workflow::iteration::IterationService;
use crate::workflow::ports::{WorkflowError, WorkflowResult};
use crate::workflow::runtime::{Artifact, TaskState};
use crate::workflow::stage::interactions as stage;

pub fn execute(
    workflow: &WorkflowConfig,
    iteration_service: &IterationService,
    task: &mut Task,
    content: &str,
    stage_name: &str,
    now: &str,
    project_root: Option<&Path>,
) -> WorkflowResult<Option<String>> {
    let artifact_name = stage::finalize_advancement::artifact_name_for_stage(
        workflow, &task.flow, stage_name, "artifact",
    );
    task.artifacts
        .set(Artifact::new(&artifact_name, content, stage_name, now));

    // Determine if an automated gate applies — either explicitly configured or derived
    // from technique checks. Agentic gates fall through to auto_advance_or_review.
    let has_automated_gate = {
        let stage_cfg = workflow.stage(&task.flow, stage_name);

        let explicit_auto = stage_cfg
            .and_then(|s| s.gate.as_ref())
            .is_some_and(|g| matches!(g, GateConfig::Automated { .. }));

        if explicit_auto {
            true
        } else if let (Some(root), Some(sc)) = (project_root, stage_cfg) {
            // Technique-derived gate: resolve to find out if any technique specifies a check.
            if sc.techniques.is_empty() {
                false
            } else {
                !load_and_resolve_techniques(root, &sc.techniques)
                    .map_err(|e| WorkflowError::InvalidState(format!("Technique resolution: {e}")))?
                    .checks
                    .is_empty()
            }
        } else {
            false
        }
    };

    if has_automated_gate {
        task.state = TaskState::awaiting_gate(stage_name);
        task.updated_at = now.to_string();
        return Ok(Some(artifact_name));
    }

    stage::auto_advance_or_review::execute(iteration_service, workflow, task, stage_name, now)?;
    Ok(Some(artifact_name))
}
