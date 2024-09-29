import { Workflow, NormalJob, Step } from 'github-actions-workflow-ts'

const RUNS_ON = 'ubuntu-latest'

const checkoutStep = new Step({
  name: 'Checkout',
  uses: 'actions/checkout@v4',
})

const setupNode = new Step({
  name: 'Setup Node',
  uses: 'actions/setup-node@v4',
  with: { 'node-version': '20' },
})

const checkWorkflow = new Step({
  name: 'Validate workflows',
  run: [
    'npm i',
    'npm run generate-workflows',
    `
    if [[ $(git diff --name-only .github/workflows/) ]]; then
      echo "Workflows are out of sync. Please regenerate them.";
      exit 1;
    fi
    `,
  ]
    .map((_) => _.trim())
    .join('\n'),
  shell: 'bash',
})

const setupRust = new Step({
  name: 'Setup Rust',
  uses: 'actions-rs/toolchain@v1',
  with: {
    profile: 'minimal',
    toolchain: 'stable',
    override: true,
  },
})

const runTests = new Step({
  name: 'Run tests',
  run: 'cargo test --workspace',
})

const wasmBuildStep = new Step({
  name: 'Build for WASM',
  run: 'cargo build --target wasm32-unknown-unknown --workspace',
})

const testJob = new NormalJob('Test', {
  'runs-on': RUNS_ON,
})

testJob.addSteps([
  checkoutStep,
  setupNode,
  checkWorkflow,
  setupRust,
  wasmBuildStep,
  runTests,
])

export const workflow = new Workflow('ci', {
  name: 'Build & Test',
  on: {
    push: {
      branches: ['main'],
    },
    pull_request: {
      branches: ['main'],
    },
  },
}).addJobs([testJob])
