import { Workflow, NormalJob, Step } from "github-actions-workflow-ts";

const checkoutStep = new Step({
  name: "Checkout",
  uses: "actions/checkout@v3",
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

testJob.addSteps([checkoutStep, setupRustStep, runTestsStep]);

export const rustTestWorkflow = new Workflow("rust-test", {
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

rustTestWorkflow.addJob(testJob);
