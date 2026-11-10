addpath(pwd);cases=jsondecode(fileread('inputs.json'));global teta1 teta2;results=struct([]);
for k=1:numel(cases)
 p=cases(k);teta1=p.observation_zenith_deg;teta2=teta1;
 para=[p.soil_moisture,p.nadir_optical_depth,p.sky_air_temperature_k,p.canopy_structure_h,p.canopy_structure_v_ratio,p.scattering_albedo_h,p.scattering_albedo_v_difference,p.roughness_height,p.roughness_mixing,p.roughness_exponent_v,p.roughness_exponent_h];
 aide=[p.frequency_ghz,p.canopy_temperature_k,p.surface_temperature_k,p.deep_temperature_k,p.bulk_density_g_cm3,p.sand_fraction,p.clay_fraction,p.temperature_moisture_scale,p.temperature_moisture_exponent,strcmp(p.surface_type,'water'),p.water_temperature_k,p.water_salinity,0];
 [tb,aux]=tauw1i(para,0,aide);
 results(k).brightness_temperature_v_k=tb(1);results(k).brightness_temperature_h_k=tb(2);
 results(k).soil_permittivity_real=aux(1);results(k).soil_permittivity_imaginary=aux(2);results(k).effective_soil_temperature_k=aux(3);
 teta1=0:80;teta2=teta1;[curve,unused]=tauw1i(para,0,aide);
 results(k).sweep_brightness_temperature_v_k=curve(1:81);results(k).sweep_brightness_temperature_h_k=curve(82:162);
end
fid=fopen('expected.json','w');fwrite(fid,jsonencode(results),'char');fclose(fid);
