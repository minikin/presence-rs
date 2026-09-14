#!/usr/bin/env bash
# PreToolUse hook: blocks `git commit` until /open-code-review:delegate-review
# has been run on the current workspace.
#
# The fingerprint covers every change a commit could pick up — tracked changes
# (staged or not) plus untracked files — which is exactly what the OCR
# workspace review reads. So `git commit -a`, `git commit <path>` and
# `git add … && git commit` cannot slip unreviewed content past it.
#
# After the review (and any fixes), record it with:
#   .claude/hooks/require-ocr-review.sh --record
# The marker lives in .git/, so it is never committed; any later change to the
# workspace invalidates it.
set -euo pipefail

cd "${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel)}"
git_dir="$(git rev-parse --git-dir 2>/dev/null)" || exit 0
marker="$git_dir/ocr-reviewed"

workspace_hash() {
    {
        git diff HEAD --binary
        git ls-files --others --exclude-standard -z |
            while IFS= read -r -d '' file; do
                shasum -a 256 "$file"
            done
    } | shasum -a 256 | cut -d' ' -f1
}

if [ "${1:-}" = "--record" ]; then
    workspace_hash > "$marker"
    echo "OCR review recorded for the current workspace."
    exit 0
fi

command="$(jq -r '.tool_input.command // empty')"
case "$command" in
    *"git commit"*) ;;
    *) exit 0 ;;
esac

if [ -f "$marker" ] && [ "$(cat "$marker")" = "$(workspace_hash)" ]; then
    exit 0
fi

reason="Commit blocked: run /open-code-review:delegate-review on the workspace first. \
After the review and any fixes, record it with \`.claude/hooks/require-ocr-review.sh --record\`, \
then retry the commit. Any change made after recording invalidates the review."

jq -n --arg r "$reason" \
    '{hookSpecificOutput: {hookEventName: "PreToolUse", permissionDecision: "deny", permissionDecisionReason: $r}}'
