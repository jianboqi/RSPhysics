// Verify original MIT R geometry, then recompute the independent thermal ODE oracle.
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const arg=name=>{const i=process.argv.indexOf(name);return i>=0?process.argv[i+1]:undefined;};
if(!arg('--source'))throw Error('Supply --source PATH/R and optionally --webr PATH/webr/dist --python PYTHON');
const source=path.resolve(arg('--source')),runtime=path.resolve(arg('--webr')??path.join(root,'tooling/reference-runner/node_modules/webr/dist'));
const folder=path.join(root,'benchmarks/reference/foursail-tir'),provenance=JSON.parse(await fs.readFile(path.join(folder,'provenance.json'),'utf8'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
for(const [name,expected]of Object.entries(provenance.source_files))if(hash(await fs.readFile(path.join(source,name)))!==expected)throw Error('Original reference hash differs: '+name);
if(process.platform==='win32'){const p=path.join(runtime,'webr-worker.js'),s=await fs.readFile(p,'utf8');await fs.writeFile(p,s.replace('await import((await import("path")).default.resolve(e))','await import((await import("node:url")).pathToFileURL((await import("path")).default.resolve(e)).href)'));}
const {WebR}=await import(pathToFileURL(path.join(runtime,'webr.mjs'))),r=new WebR();await r.init();
const temporary=await fs.mkdtemp(path.join(source,'thermal-verification-'));
try{
 for(const name of ['campbell.R','volscatt.R','jfunc1.R','jfunc2.R','jfunc3.R','dcum.R','dladgen.R','fourSAIL.R']){
  let text=await fs.readFile(path.join(source,'R',name),'utf8');
  // Observe already-calculated geometry; no reference equation is changed.
  if(name==='fourSAIL.R')text=text.replace('sigb <- ddb*rho+ddf*tau','return(c(ks,ko,bf,sob,sof,dso))\n  sigb <- ddb*rho+ddf*tau');
  await r.FS.writeFile('/tmp/'+name,new TextEncoder().encode(text));await r.evalRVoid(`source('/tmp/${name}')`);
 }
 const cases=JSON.parse(await fs.readFile(path.join(folder,'cases.json'),'utf8')),queue=cases.map(c=>c.parameters),base=cases[0].parameters;
 for(let a=-85;a<=85;a++)queue.push({...base,view_zenith_deg:Math.abs(a),relative_azimuth_deg:a<0?180:0});
 for(let z=0;z<=85;z+=5)for(let az=0;az<360;az+=10)queue.push({...base,view_zenith_deg:z,relative_azimuth_deg:az>180?360-az:az});
 const geometry=[];
 for(const p of queue){const v=p.leaf_angle_distribution==='verhoef';geometry.push(await(await r.evalR(`fourSAIL(list(reflectance=.02,transmittance=0),type_lidf=${v?1:2},lidf_a=${v?p.lidf_a:p.mean_leaf_angle_deg},lidf_b=${p.lidf_b},lai=${p.leaf_area_index},hotspot=${p.hotspot},tts=${p.solar_zenith_deg},tto=${p.view_zenith_deg},psi=${p.relative_azimuth_deg},rsoil=.05)`)).toArray());}
 const file=path.join(temporary,'geometry.json'),out=path.join(temporary,'actual.json');await fs.writeFile(file,JSON.stringify({parameters:queue,geometry}));
 const result=spawnSync(arg('--python')??'python',[path.join(folder,'generate_reference.py'),'--geometry',file,'--out',out],{cwd:root,encoding:'utf8'});
 if(result.status!==0)throw Error(result.stderr||result.stdout);process.stdout.write(result.stdout);
 const actual=JSON.parse(await fs.readFile(out,'utf8')),want=JSON.parse(await fs.readFile(path.join(folder,'expected.json'),'utf8'));let max=0,count=0;
 function compare(a,b,label){if(typeof b==='number'){const d=Math.abs(a-b);if(!Number.isFinite(a)||d>2e-9+2e-10*Math.abs(b))throw Error(label+': '+a+' vs '+b);max=Math.max(max,d);count++;}else if(Array.isArray(b)){if(a.length!==b.length)throw Error('shape '+label);b.forEach((v,i)=>compare(a[i],v,label+'/'+i));}else if(b&&typeof b==='object'){for(const k of Object.keys(b))compare(a[k],b[k],label+'/'+k);}else if(a!==b)throw Error('value '+label);}
 compare(actual.cases,want.cases,'cases');compare(actual.default_arrays,want.default_arrays,'arrays');console.log(`Original R + independent thermal ODE: ${cases.length} cases, ${count} values, max error ${max}`);
}finally{
 await r.close();
 const resolved=path.resolve(temporary);
 if(path.dirname(resolved)!==source||!path.basename(resolved).startsWith('thermal-verification-'))throw Error('Unsafe temporary reference directory');
 await fs.rm(resolved,{recursive:true,force:true});
}
