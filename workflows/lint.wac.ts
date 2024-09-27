
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
    components: "rustfmt, clippy",
  },
});

const runRustfmtStep = new Step({
  name: "Run rustfmt",
  run: "cargo fmt --all -- --check",
});

const runClippyStep = new Step({
  name: "Run clippy",
  run: "cargo clippy --all-targets --all-features -- -D warnings",
});

const lintJob = new NormalJob("Lint", {
  "runs-on": "ubuntu-latest",
});

lintJob.addSteps([checkoutStep, setupRustStep, runRustfmtStep, runClippyStep]);

export const rustLintWorkflow = new Workflow("rust-lint", {
  name: "Rust Lint",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
});

rustLintWorkflow.addJob(lintJob);
