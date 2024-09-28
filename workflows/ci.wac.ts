import { Workflow, NormalJob, Step } from "github-actions-workflow-ts";

const checkoutStep = new Step({
  name: "Checkout",
  uses: "actions/checkout@v4",
});

const setupNode = new Step({
  name: "Setup Node",
  uses: "actions/setup-node@v4",
  with: { "node-version": "20" },
});

const validateWorkflows = new Step({
  name: "Validate workflows",
  run: `
    npm i
    npm run generate-workflows
    if [[ $(git diff --name-only .github/workflows/) ]]; then
      echo "Workflows are out of sync. Please regenerate them."
      exit 1
    fi
  `,
  shell: "bash",
});

const setupRustStep = new Step({
  name: "Setup Rust",
  uses: "actions-rs/toolchain@v1",
  with: {
    profile: "minimal",
    toolchain: "stable",
    override: true,
  },
});

const runTestsStep = new Step({
  name: "Run tests",
  run: "cargo test --workspace",
});

const testJob = new NormalJob("Test", {
  "runs-on": "ubuntu-latest",
});

testJob.addSteps([
  checkoutStep,
  setupNode,
  validateWorkflows,
  setupRustStep,
  runTestsStep,
]);

export const mainWorkflow = new Workflow("CI", {
  name: "Rust Test",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
});

mainWorkflow.addJob(testJob);
