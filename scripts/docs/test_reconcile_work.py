import unittest
from pathlib import Path

from scripts.docs import reconcile_work

sync_roadmap = reconcile_work.sync_roadmap


def issue(number, state="open", labels=(), assignees=(), body="", title=None):
    return {
        "number": number,
        "title": title or f"Issue {number}",
        "html_url": f"https://example.test/{number}",
        "state": state,
        "labels": [{"name": value} for value in labels],
        "assignees": [{"login": value} for value in assignees],
        "milestone": {"number": 1, "title": "M1"},
        "updated_at": "2026-07-25T00:00:00Z",
        "body": body,
    }


class ReconcileWorkTests(unittest.TestCase):
    def test_valid_leaf_packet_and_epic_without_packet(self):
        issues = [
            issue(1, labels=("roadmap-task", "status:in-progress", "size:S"), assignees=("dev",)),
            issue(2, labels=("roadmap-task", "status:in-progress", "size:epic"), assignees=("dev",)),
        ]
        packets = [reconcile_work.Packet(Path("issue-0001.md"), 1)]
        index = reconcile_work.render_active_index("---\n---\n", packets, {item["number"]: item for item in issues})
        errors, _ = reconcile_work.validate(issues, packets, index, sync_roadmap.render(issues))
        self.assertEqual(errors, [])

    def test_closed_packet_and_missing_leaf_packet_are_errors(self):
        issues = [
            issue(1, state="closed", labels=("roadmap-task", "size:S")),
            issue(2, labels=("roadmap-task", "status:in-progress", "size:S"), assignees=("dev",)),
        ]
        packets = [reconcile_work.Packet(Path("issue-0001.md"), 1)]
        index = reconcile_work.render_active_index("---\n---\n", packets, {item["number"]: item for item in issues})
        errors, _ = reconcile_work.validate(issues, packets, index, sync_roadmap.render(issues))
        self.assertTrue(any("closed issue #1" in value for value in errors))
        self.assertTrue(any("issue #2 has no active packet" in value for value in errors))

    def test_empty_depends_line_does_not_capture_blocks_line(self):
        item = issue(1, body="Depends on:\n\nBlocks: #99")
        self.assertEqual(reconcile_work.dependencies(item), [])

    def test_blocked_issue_with_closed_dependencies_is_stale(self):
        issues = [
            issue(1, state="closed", labels=("roadmap-task", "size:S")),
            issue(
                2,
                labels=("roadmap-task", "status:blocked", "size:S"),
                body="Depends on: #1",
            ),
        ]
        index = reconcile_work.render_active_index("---\n---\n", [], {item["number"]: item for item in issues})
        errors, _ = reconcile_work.validate(issues, [], index, sync_roadmap.render(issues))
        self.assertIn("blocked issue #2 has no unresolved dependencies", errors)

    def test_empty_active_index_is_generated_deterministically(self):
        rendered = reconcile_work.render_active_index("---\ntitle: Active\n---\n", [], {})
        self.assertIn("## Active", rendered)
        self.assertIn("_No active packets.", rendered)

    def test_github_mutations_are_explicit_and_dependency_aware(self):
        issues = [
            issue(1, state="closed", labels=("roadmap-task", "status:in-progress", "size:S")),
            issue(2, state="closed", labels=("roadmap-task", "size:S")),
            issue(3, labels=("roadmap-task", "status:blocked", "size:S"), body="Depends on: #1 #2"),
            issue(4, labels=("roadmap-task", "status:blocked", "size:S"), body="Depends on: #5"),
            issue(5, labels=("roadmap-task", "status:ready", "size:S")),
        ]

        commands = reconcile_work.github_commands(issues)

        self.assertEqual(len(commands), 2)
        self.assertIn("--remove-label", commands[0])
        self.assertIn("status:in-progress", commands[0])
        self.assertIn("--add-label", commands[1])
        self.assertIn("status:ready", commands[1])

    def test_title_drift_warns_without_blocking_structural_check(self):
        issues = [
            issue(
                1,
                labels=("roadmap-task", "status:in-progress", "size:S"),
                assignees=("dev",),
                title="New [title]",
            )
        ]
        packet = reconcile_work.Packet(Path("issue-0001.md"), 1)
        current_index = (
            "---\n---\n\n## Active\n\n"
            "- [Issue #1 — Old title](issue-0001.md)\n"
        )

        errors, warnings = reconcile_work.validate(
            issues, [packet], current_index, sync_roadmap.render(issues)
        )

        self.assertEqual(errors, [])
        self.assertTrue(any("titles differ" in value for value in warnings))

    def test_open_non_active_packet_is_reported_but_not_a_document_fix_candidate(self):
        issues = [issue(1, labels=("roadmap-task", "status:ready", "size:S"))]
        packet = reconcile_work.Packet(Path("issue-0001.md"), 1)
        index = reconcile_work.render_active_index("---\n---\n", [packet], {1: issues[0]})

        errors, _ = reconcile_work.validate(
            issues, [packet], index, sync_roadmap.render(issues)
        )

        self.assertTrue(any("packet #1 exists" in value for value in errors))


if __name__ == "__main__":
    unittest.main()
