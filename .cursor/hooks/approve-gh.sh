#!/bin/bash
# Require user approval before any GitHub CLI (gh) command runs.

input=$(cat)
command=$(echo "$input" | jq -r '.command // empty')

if echo "$command" | grep -qE '\bgh\b'; then
  echo '{
    "permission": "ask",
    "user_message": "This command uses the GitHub CLI (gh). Approve to continue.",
    "agent_message": "User policy requires approval before running gh."
  }'
  exit 0
fi

echo '{ "permission": "allow" }'
exit 0
