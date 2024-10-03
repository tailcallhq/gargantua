import { Workflow, NormalJob, Step } from "github-actions-workflow-ts";

const MACHINE = "ubuntu-latest";

const checkoutStep = new Step({
  name: "Checkout",
  uses: "actions/checkout@v4",
});

const setupNode = new Step({
  name: "Setup Node",
  uses: "actions/setup-node@v4",
  with: { "node-version": "20" },
});

const checkWorkflow = new Step({
  name: "Validate Workflows",
  run: ["npm run build", "npm run check-workflows"]
    .map((_) => _.trim())
    .join("\n"),
});

const runRustfmtStep = new Step({
  name: "Run rustfmt",
  run: "cargo fmt --all -- --check",
});

const runClippyStep = new Step({
  name: "Run clippy",
  run: "cargo clippy --all -- -D warnings",
});

const runTests = new Step({
  name: "Run Tests",
  run: "cargo test --workspace",
});

const runPrettier = new Step({
  name: "Run Prettier",
  run: "npm run prettier:check",
});

const wasmBuildJob = new NormalJob("WASM", {
  "runs-on": MACHINE,
}).addSteps([
  new Step({
    run: "rustup target add wasm32-unknown-unknown",
  }),
  new Step({
    run: "cargo build --target wasm32-unknown-unknown --workspace",
  }),
]);

const workspaceLintJob = new NormalJob("Test", {
  "runs-on": MACHINE,
}).addSteps([checkoutStep, runTests]);

const workspaceTestJob = new NormalJob("Format", {
  "runs-on": MACHINE,
}).addSteps([
  checkoutStep,
  setupNode,
  runPrettier,
  runRustfmtStep,
  runClippyStep,
]);

// Workflow validation job
const workflowValidateJob = new NormalJob("Validate", {
  "runs-on": MACHINE,
}).addSteps([checkoutStep, setupNode, checkWorkflow]);

export const workflow = new Workflow("ci", {
  name: "CI",
  on: {
    push: {
      branches: ["main"],
    },
    pull_request: {
      branches: ["main"],
    },
  },
}).addJobs([
  workspaceLintJob,
  workspaceTestJob,
  wasmBuildJob,
  workflowValidateJob,
]);
