import * as fs from "fs/promises";
import { workflow as mainWorkflow } from "./workflow";
import * as yml from "js-yaml";
import * as path from "path";

async function generateWorkflow() {
  const workflowYaml = mainWorkflow.workflow;
  const content = yml.dump(workflowYaml);
  const loc = path.resolve(`.github/workflows/${mainWorkflow?.filename}.yml`);

  await fs.writeFile(loc, content);
  console.log("Workflow generated at", loc);
}

async function checkWorkflow() {
  const workflowYaml = mainWorkflow.workflow;
  const expected = yml.dump(workflowYaml);
  const actual = await fs.readFile(
    `.github/workflows/${mainWorkflow?.filename}.yml`,
  );
  if (expected !== actual.toString()) {
    throw "Workflows are out of sync! Please regenerate them using `npm run generate-workflows`.";
  } else {
    console.log("Workflows are ok!");
  }
}

async function main() {
  const args = process.argv.slice(2);
  if (args[0] === "check") {
    await checkWorkflow();
  } else if (args[0] === "generate") {
    await generateWorkflow();
  } else {
    throw "Invalid command: " + args[0];
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
