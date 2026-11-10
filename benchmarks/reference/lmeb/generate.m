% Original L-MEB forward reference. Run from the repository root with MATLAB.
root=fileparts(fileparts(fileparts(fileparts(mfilename('fullpath')))));
source=fullfile(root,'.reference-cache','lmeb','LMEB');
addpath(source);
cases=jsondecode(fileread(fullfile(root,'benchmarks','reference','lmeb','inputs.json')));
global teta1 teta2;
results=struct([]);
for k=1:numel(cases)
 p=cases(k).parameters;
 teta1=p.observation_zenith_deg; teta2=teta1;
 para=[p.soil_moisture,p.nadir_optical_depth,p.sky_air_temperature_k,p.canopy_structure_h,p.canopy_structure_v_ratio,p.scattering_albedo_h,p.scattering_albedo_v_difference,p.roughness_height,p.roughness_mixing,p.roughness_exponent_v,p.roughness_exponent_h];
 aide=[1.41,p.canopy_temperature_k,p.surface_temperature_k,p.deep_temperature_k,p.bulk_density_g_cm3,p.sand_fraction,p.clay_fraction,p.temperature_moisture_scale,p.temperature_moisture_exponent,0,300,0,0];
 [tb,unused]=tauw1i(para,0,aide);
 results(k).name=cases(k).name;
 results(k).brightness_temperature_v_k=tb(1);
 results(k).brightness_temperature_h_k=tb(2);
 teta1=0:80;teta2=teta1;
 [curve,unused]=tauw1i(para,0,aide);
 results(k).sweep_brightness_temperature_v_k=curve(1:81);
 results(k).sweep_brightness_temperature_h_k=curve(82:162);
end
out=struct('runtime',version,'frequency_ghz',1.41,'mode','original tauw1i forward soil branch; unchanged scientific functions','cases',results);
fid=fopen(fullfile(root,'benchmarks','reference','lmeb','expected.json'),'w');
fprintf(fid,'%s\n',jsonencode(out,PrettyPrint=true));fclose(fid);
disp('L-MEB original references generated.');
