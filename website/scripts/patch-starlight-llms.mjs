import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

const path = resolve('node_modules/starlight-llms-txt/llms-full.txt.ts');
const original = await readFile(path, 'utf8');
const importLine = "import { starlightLllmsTxtContext } from 'virtual:starlight-llms-txt/context';";
const excludeLine = '\t\texclude: starlightLllmsTxtContext.exclude,';

if (original.includes(importLine) && original.includes(excludeLine)) process.exit(0);

const withImport = original.replace(
  "import { generateLlmsTxt } from './generator';",
  `${importLine}\nimport { generateLlmsTxt } from './generator';`,
);
const patched = withImport.replace(
  '\t\tdescription: `This is the full developer documentation for ${getSiteTitle()}`,',
  '\t\tdescription: `This is the full developer documentation for ${getSiteTitle()}`,\n' + excludeLine,
);

if (patched === original || !patched.includes(importLine) || !patched.includes(excludeLine)) {
  throw new Error('starlight-llms-txt changed: update the stable-only llms-full compatibility patch');
}

await writeFile(path, patched, 'utf8');
