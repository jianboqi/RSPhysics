/** Execute both original Fortran programs through their Emscripten modules.
 * Usage: node tooling/sixs/run-wasm.mjs BUILD_DIR INPUTS_JSON OUTPUT_JSON
 * A fresh module instance and filesystem are used for each run.
 */
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const [buildDir, inputsFile, outputFile] = process.argv.slice(2);
if (!buildDir || !inputsFile || !outputFile) throw new Error('Expected BUILD_DIR INPUTS_JSON OUTPUT_JSON');
const cases = JSON.parse(await readFile(inputsFile, 'utf8'));
const results = {};
for (const entry of ['sixsV2.1', 'sixsV2.1-lutaero']) {
  const { default: createModule } = await import(pathToFileURL(resolve(buildDir, entry+'.mjs')).href);
  for (const [name, inputCards] of Object.entries(cases)) {
    const input = new TextEncoder().encode(inputCards);
    let cursor = 0;
    const stdout = [], stderr = [];
    const start = performance.now();
    const instance = await createModule({
      stdin: () => cursor < input.length ? input[cursor++] : null,
      print: line => stdout.push(line), printErr: line => stderr.push(line),
    });
    try { instance.callMain([]); }
    catch (error) { if (error.status !== 0) throw error; }
    const report = stdout.join('\n')+'\n';
    if (!report.includes('global gas. trans.')) throw new Error(`Incomplete ${name}/${entry}: ${stderr.join('\n')}`);
    results[name+'/'+entry] = { report, stderr: stderr.join('\n'), elapsed_s: (performance.now()-start)/1000 };
    console.log(name, entry, results[name+'/'+entry].elapsed_s);
  }
}
await writeFile(outputFile, JSON.stringify(results, null, 2)+'\n');
