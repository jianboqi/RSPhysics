// Original R fourSAIL, all nine outputs and both 13-bin LIDFs.
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {inflateSync} from 'node:zlib';
import {createHash} from 'node:crypto';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const arg=name=>{const i=process.argv.indexOf(name);return i>=0?process.argv[i+1]:undefined;};
if(!arg('--source'))throw Error('Usage: node verify_prosail_full.mjs --source PATH/author-prosail [--webr PATH/webr/dist]');
const source=path.resolve(arg('--source')),runtime=path.resolve(arg('--webr')??path.join(root,'tooling/reference-runner/node_modules/webr/dist'));
const folder=path.join(root,'benchmarks/reference/prosail-d');
const provenance=JSON.parse(await fs.readFile(path.join(folder,'upstream-provenance.json'),'utf8'));
const full=JSON.parse(await fs.readFile(path.join(folder,'full-provenance.json'),'utf8'));
const hash=b=>createHash('sha256').update(b).digest('hex');
for(const [name,want]of Object.entries(full.files))if(hash(await fs.readFile(path.join(folder,name)))!==want)throw Error('Fixture checksum mismatch: '+name);
if(process.platform==='win32'){
 const file=path.join(runtime,'webr-worker.js'),text=await fs.readFile(file,'utf8');
 await fs.writeFile(file,text.replace('await import((await import("path")).default.resolve(e))','await import((await import("node:url")).pathToFileURL((await import("path")).default.resolve(e)).href)'));
}
const {WebR}=await import(pathToFileURL(path.join(runtime,'webr.mjs'))),r=new WebR();await r.init();
try{
 for(const name of ['campbell.R','volscatt.R','jfunc1.R','jfunc2.R','jfunc3.R','dcum.R','dladgen.R','fourSAIL.R']){
  const bytes=await fs.readFile(path.join(source,'R',name));if(hash(bytes)!==provenance.files['R/'+name].sha256)throw Error('Original source checksum mismatch: '+name);
  await r.FS.writeFile('/tmp/'+name,bytes);await r.evalRVoid(`source('/tmp/${name}')`);
 }
 await r.FS.writeFile('/tmp/soil.tsv',await fs.readFile(path.join(root,'crates/models/vegetation/prosail-d/data/soil.tsv')));
 await r.evalRVoid(`soil<-read.table('/tmp/soil.tsv',header=TRUE,sep='\t')`);
 const cases=JSON.parse(inflateSync(await fs.readFile(path.join(folder,'full-leaf-inputs.json.zlib')))),expected=JSON.parse(inflateSync(await fs.readFile(path.join(folder,'full-expected.json.zlib')))).cases;
 const fields={rdot:'hemispherical_directional_reflectance',rsot:'brf',rddt:'bihemispherical_reflectance',rsdt:'hemispherical_reflectance',fcover:'vegetation_cover',abs_dir:'direct_absorptance',abs_hem:'diffuse_absorptance',rsdstar:'direct_albedo',rddstar:'diffuse_albedo'};
 let maximum=0,samples=0;
 for(let i=0;i<cases.length;i++){
  const c=cases[i],p=c.parameters,verhoef=p.leaf_angle_distribution==='verhoef';
  await r.evalRVoid(`canopy<-fourSAIL(lop=list(reflectance=c(${c.reflectance.join(',')}),transmittance=c(${c.transmittance.join(',')})),type_lidf=${verhoef?1:2},lidf_a=${verhoef?p.lidf_a:p.mean_leaf_angle_deg},lidf_b=${p.lidf_b??-.15},lai=${p.leaf_area_index},hotspot=${p.hotspot},tts=${p.solar_zenith_deg},tto=${p.view_zenith_deg},psi=${p.relative_azimuth_deg},rsoil=${p.soil_brightness}*(${p.soil_dry_fraction}*soil$Dry_Soil+(1-${p.soil_dry_fraction})*soil$Wet_Soil))`);
  for(const [key,name]of Object.entries(fields)){
   const values=await(await r.evalR(`canopy$${key}`)).toArray(),want=expected[i].result[name];
   for(let j=0;j<want.length;j++){const got=values.length===1?values[0]:values[j],difference=Math.abs(got-want[j]);
    if(!Number.isFinite(got)||difference>2e-10+2e-8*Math.abs(want[j]))throw Error(`${i}/${name}/${j}: ${got} vs ${want[j]}`);
    maximum=Math.max(maximum,difference);samples++;
   }
  }
 }
 console.log(`Original R fourSAIL: ${cases.length} complete cases, ${samples} values, maximum error ${maximum}`);
}finally{await r.close();}
