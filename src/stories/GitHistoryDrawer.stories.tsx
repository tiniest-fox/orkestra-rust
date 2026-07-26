// Storybook stories for GitHistoryDrawer — default state and subtask provenance badge.
import type { Meta, StoryObj } from "@storybook/react";
import { GitHistoryDrawer } from "../components/Feed/GitHistoryDrawer";
import { createMockTransport, StorybookProviders } from "./storybook-helpers";

// ============================================================================
// Helpers
// ============================================================================

function makeTransport(
  commits: Array<{
    hash: string;
    message: string;
    body: string | null;
    author: string;
    timestamp: string;
    file_count: number | null;
    from_subtask: { parent_id: string; subtask_id: string; title: string } | null;
  }>,
) {
  const base = createMockTransport();
  return {
    ...base,
    call: <T,>(method: string, params?: Record<string, unknown>): Promise<T> => {
      if (method === "get_commit_log") {
        return Promise.resolve(commits as T);
      }
      return base.call<T>(method, params);
    },
  };
}

// ============================================================================
// Stories
// ============================================================================

const meta = {
  title: "Feed/GitHistoryDrawer",
  parameters: { layout: "fullscreen" },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

/** Empty commit list — no subtask provenance. */
export const Empty: Story = {
  render: () => (
    <StorybookProviders transport={makeTransport([])}>
      <div className="relative w-[720px] h-[560px] bg-surface">
        <GitHistoryDrawer onClose={() => {}} />
      </div>
    </StorybookProviders>
  ),
};

/** Commit list with mixed provenance — one sub-Trak commit, one regular commit. */
export const WithSubtaskProvenance: Story = {
  render: () => (
    <StorybookProviders
      transport={makeTransport([
        {
          hash: "abc1234",
          message: "Implement feature flag store",
          body: null,
          author: "Alice Developer",
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          file_count: 4,
          from_subtask: {
            parent_id: "parent-task-id",
            subtask_id: "sub-abc1234",
            title: "Add feature flag store implementation",
          },
        },
        {
          hash: "def5678",
          message: "Fix login redirect after session expiry",
          body: null,
          author: "Bob Engineer",
          timestamp: new Date(Date.now() - 7200000).toISOString(),
          file_count: 2,
          from_subtask: null,
        },
      ])}
    >
      <div className="relative w-[720px] h-[560px] bg-surface">
        <GitHistoryDrawer onClose={() => {}} />
      </div>
    </StorybookProviders>
  ),
};
