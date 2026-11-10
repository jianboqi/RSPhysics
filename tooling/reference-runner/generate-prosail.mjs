import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import crypto from 'node:crypto';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const runtime=path.join(root,'.reference-cache/prospect-d/node_modules/webr/dist');
if(process.platform==='win32'){
  const worker=path.join(runtime,'webr-worker.js');const source=await fs.readFile(worker,'utf8');
  const before='await import((await import("path")).default.resolve(e))';
  const after='await import((await import("node:url")).pathToFileURL((await import("path")).default.resolve(e)).href)';
  if(source.includes(before))await fs.writeFile(worker,source.replace(before,after));
  else if(!source.includes(after))throw new Error('Review the changed WebR Windows loader');
}
const {WebR}=await import(pathToFileURL(path.join(runtime,'webr.mjs')));
const r=new WebR();await r.init();
try{
  await r.installPackages(['expint']);
  const expintVersion=await r.evalRString('as.character(packageVersion("expint"))');
  if(expintVersion!=='0.2.1')throw new Error('Require the recorded expint 0.2.1');
  const leafManifest=JSON.parse(await fs.readFile(path.join(root,'benchmarks/reference/prospect-d/upstream-provenance.json'),'utf8'));
  const manifest=JSON.parse(await fs.readFile(path.join(root,'benchmarks/reference/prosail-d/upstream-provenance.json'),'utf8'));
  async function put(model,name,record){const bytes=await fs.readFile(path.join(root,`.reference-cache/${model}/upstream`,name));if(crypto.createHash('sha256').update(bytes).digest('hex')!==record.sha256)throw new Error('Hash mismatch '+name);await r.FS.writeFile('/tmp/'+path.basename(name),bytes);}
  for(const name of ['R/Lib_PROSPECT.R','data-raw/dataSpec_PRO.txt'])await put('prospect-d',name,leafManifest.files[name]);
  const functions=['campbell.R','volscatt.R','jfunc1.R','jfunc2.R','jfunc3.R','fourSAIL.R'];
  for(const file of functions)await put('prosail-d','R/'+file,manifest.files['R/'+file]);
  await put('prosail-d','data-raw/dataSpec_SOIL_ATM.txt',manifest.files['data-raw/dataSpec_SOIL_ATM.txt']);
  await r.evalRVoid(`library(expint);source('/tmp/Lib_PROSPECT.R');define_Input_PROSPECT<-function(Input_PROSPECT,...) Input_PROSPECT;sp<-read.table('/tmp/dataSpec_PRO.txt',header=TRUE,sep='\\t');sp$calctav_40<-calctav(40,sp$nrefrac);sp$calctav_90<-calctav(90,sp$nrefrac);soil<-read.table('/tmp/dataSpec_SOIL_ATM.txt',header=TRUE,sep='\\t')`);
  for(const file of functions)await r.evalRVoid(`source('/tmp/${file}')`);
  const inputs=JSON.parse(await fs.readFile(path.join(root,'benchmarks/reference/prosail-d/inputs.json'),'utf8'));
  const output={suite:'prosail-d-author-r-v1',canopy_upstream_commit:manifest.commit,leaf_upstream_commit:leafManifest.commit,
    soil_sha256:manifest.files['data-raw/dataSpec_SOIL_ATM.txt'].sha256,leaf_coefficients_sha256:leafManifest.files['data-raw/dataSpec_PRO.txt'].sha256,
    interpreter:await r.evalRString('R.version.string'),expint_version:expintVersion,webr_version:'0.6.0',
    scientific_configuration:'PROSPECT-D + author 4SAIL, 13-bin Campbell LIDF, direct illumination, original dry/wet soil blend',
    tolerance:{absolute:2e-10,relative:2e-8},wavelengths_nm:await(await r.evalR('sp$lambda')).toArray(),cases:[]};
  const leafMap={n:'N',chlorophyll:'CHL',carotenoids:'CAR',anthocyanins:'ANT',brown_pigments:'BROWN',water:'EWT',dry_matter:'LMA',incidence_angle_deg:'alpha'};
  for(const item of inputs){
    const p=item.parameters;const args=Object.entries(leafMap).map(([key,name])=>`${name}=${p[key]}`).join(',');
    await r.evalRVoid(`leaf<-suppressWarnings(PROSPECT(SpecPROSPECT=sp,Input_PROSPECT=data.frame(${args},PROT=0,CBC=0)));rsoil<-${p.soil_brightness}*(${p.soil_dry_fraction}*soil$Dry_Soil+(1-${p.soil_dry_fraction})*soil$Wet_Soil)`);
    if(p.leaf_area_index===0){
      // The pinned R fourSAIL has undefined 0/0 hotspot arithmetic at LAI=0.
      // Record this as a separate analytical bare-soil limit, never as upstream output.
      await r.evalRVoid('canopy<-list(rsot=rsoil,rsdt=rsoil)');
    }else{
      await r.evalRVoid(`canopy<-suppressWarnings(fourSAIL(lop=list(reflectance=leaf$Reflectance,transmittance=leaf$Transmittance),type_lidf=2,lidf_a=${p.mean_leaf_angle_deg},lai=${p.leaf_area_index},hotspot=${p.hotspot},tts=${p.solar_zenith_deg},tto=${p.view_zenith_deg},psi=${p.relative_azimuth_deg},rsoil=rsoil))`);
    }
    const brf=await(await r.evalR('canopy$rsot')).toArray();const hemispherical_reflectance=await(await r.evalR('canopy$rsdt')).toArray();
    if([...brf,...hemispherical_reflectance].some(x=>!Number.isFinite(x)))throw new Error('Nonfinite original output: '+item.name);
    output.cases.push({...item,brf,hemispherical_reflectance});console.log('R PROSAIL: '+item.name);
  }
  await fs.writeFile(path.join(root,'benchmarks/reference/prosail-d/expected.json'),JSON.stringify(output)+'\n');
  await fs.writeFile(path.join(root,'.reference-cache/prosail-d/reference.json'),JSON.stringify(output)+'\n');
  console.log('Saved '+output.cases.length+' explicitly sourced PROSAIL-D cases');
}finally{await r.close();}
