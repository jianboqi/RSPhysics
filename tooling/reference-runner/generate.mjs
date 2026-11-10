// Execute the authors' unchanged PROSPECT/calctav functions using an independent R interpreter.
// All runtime downloads, upstream source and intermediate files live in .reference-cache.
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import crypto from 'node:crypto';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const cache = path.join(root, '.reference-cache/prospect-d');
const suite = path.join(root, 'benchmarks/reference/prospect-d');
// WebR 0.6.0 resolves a Windows filesystem path and passes it directly to import().
// Convert only that worker-loader path to a file URL. No R/math code is modified.
if (process.platform === 'win32') {
  const worker = path.join(cache, 'node_modules/webr/dist/webr-worker.js');
  const original = await fs.readFile(worker, 'utf8');
  const before = 'await import((await import("path")).default.resolve(e))';
  const after = 'await import((await import("node:url")).pathToFileURL((await import("path")).default.resolve(e)).href)';
  if (original.includes(before)) await fs.writeFile(worker, original.replace(before, after));
  else if (!original.includes(after)) throw new Error('WebR worker loader changed; review Windows adaptation');
}
const { WebR } = await import(pathToFileURL(path.join(cache, 'node_modules/webr/dist/webr.mjs')));
const inputs = JSON.parse(await fs.readFile(path.join(suite, 'inputs.json'), 'utf8'));
const provenance = JSON.parse(await fs.readFile(path.join(suite, 'upstream-provenance.json'), 'utf8'));
const upstream = path.join(cache, 'upstream');
const webR = new WebR();
await webR.init();
try {
  await webR.installPackages(['expint']);
  if (await webR.evalRString('as.character(packageVersion("expint"))') !== '0.2.1') {
    throw new Error('Reference expint version changed; reproduce with recorded 0.2.1 before updating fixtures');
  }
  for (const name of ['R/Lib_PROSPECT.R', 'data-raw/dataSpec_PRO.txt']) {
    const bytes = await fs.readFile(path.join(upstream, name));
    if (crypto.createHash('sha256').update(bytes).digest('hex') !== provenance.files[name].sha256) {
      throw new Error(`Upstream hash mismatch: ${name}`);
    }
    await webR.FS.writeFile(`/tmp/${path.basename(name)}`, bytes);
  }
  await webR.evalRVoid(`
    library(expint)
    source('/tmp/Lib_PROSPECT.R')
    # Supply a complete, explicitly versioned parameter frame. This harness replaces only
    # the package-dependent input normalizer. PROSPECT and calctav remain unchanged.
    define_Input_PROSPECT <- function(Input_PROSPECT, ...) Input_PROSPECT
    sp <- read.table('/tmp/dataSpec_PRO.txt', header=TRUE, sep='\\t')
    sp$calctav_40 <- calctav(40, sp$nrefrac)
    sp$calctav_90 <- calctav(90, sp$nrefrac)
  `);
  const output = {
    suite: 'prospect-d-author-r-v1',
    upstream_commit: provenance.commit,
    source_sha256: provenance.files['R/Lib_PROSPECT.R'].sha256,
    coefficients_sha256: provenance.files['data-raw/dataSpec_PRO.txt'].sha256,
    interpreter: await webR.evalRString('R.version.string'),
    expint_version: await webR.evalRString('as.character(packageVersion("expint"))'),
    webr_version: '0.6.0',
    input_adapter: 'Complete parameter frame; bypass only define_Input_PROSPECT normalization; PROT=CBC=0. Scientific PROSPECT/calctav function bodies unchanged.',
    tolerance: {absolute: 2e-10, relative: 2e-8},
    wavelengths_nm: await (await webR.evalR('sp$lambda')).toArray(),
    cases: [],
  };
  const mapping = {n:'N', chlorophyll:'CHL', carotenoids:'CAR', anthocyanins:'ANT', brown_pigments:'BROWN', water:'EWT', dry_matter:'LMA', incidence_angle_deg:'alpha'};
  for (const entry of inputs) {
    const args = Object.entries(mapping).map(([key, rkey]) => {
      const value = entry.parameters[key];
      if (typeof value !== 'number' || !Number.isFinite(value)) throw new Error(`Invalid input ${key}`);
      return `${rkey}=${value}`;
    }).join(',');
    await webR.evalRVoid(`out <- suppressWarnings(PROSPECT(SpecPROSPECT=sp, Input_PROSPECT=data.frame(${args},PROT=0,CBC=0)))`);
    const reflectance = await (await webR.evalR('out$Reflectance')).toArray();
    const transmittance = await (await webR.evalR('out$Transmittance')).toArray();
    if ([...reflectance, ...transmittance].some(x => !Number.isFinite(x))) throw new Error(`Non-finite upstream output: ${entry.name}`);
    output.cases.push({...entry, reflectance, transmittance});
    console.log(`R reference: ${entry.name}`);
  }
  await fs.writeFile(path.join(cache, 'reference.json'), JSON.stringify(output) + '\n');
  // Compact, offline regression oracle; unlike the R installation this stays with the tests.
  await fs.writeFile(path.join(suite, 'expected.json'), JSON.stringify(output) + '\n');
  console.log(`Saved ${output.cases.length} full-spectrum cases from ${output.interpreter}`);
} finally {
  await webR.close();
}
