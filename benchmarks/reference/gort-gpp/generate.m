% Pointwise mvp=1/flagSeason=0 path from Main4PSN, without Excel/site I/O.
% Original scientific function bodies are unmodified.
root=fileparts(fileparts(fileparts(fileparts(mfilename('fullpath')))));
source=fullfile(root,'.reference-cache','gort-gpp','source');
addpath(source);
cases=jsondecode(fileread(fullfile(root,'benchmarks','reference','gort-gpp','inputs.json')));
results=struct([]);
for k=1:numel(cases)
 p=cases(k).parameters;
 P=InitialDefaultPara();
 P.x=p.leaf_angle_parameter; P.pa=p.pressure_pa;
 P.Vcmax0a=p.vcmax25_young; P.Vcmax1_3a=p.vcmax25_mature;
 P.par_omega=p.leaf_single_scattering_albedo; P.par_refl=p.background_reflectance;
 P.g0=p.minimum_stomatal_conductance;
 crown=struct('h1',p.crown_base_height_m,'h2',p.crown_top_height_m,'R',p.crown_radius_m,'lambda',p.crown_density_m2,'brratio',p.crown_axis_ratio,'lai',p.leaf_area_index);
 gaps=compute_gap_fractions(crown,P,1);
 zenith=p.solar_zenith_deg*P.PI/180;
 costheta=cos(zenith);
 Kb=sqrt(P.x*P.x+tan(zenith)^2)/(P.x+1.774*(P.x+1.182)^(-0.733));
 % Preserve the Main4PSN 1-based rounded index (unlike radiation's +1).
 idx=int16(p.solar_zenith_deg);
 notgap=1-gaps.Kg_gaps(idx)-(1-gaps.Kg_gaps(idx))*gaps.Kz_gaps(idx);
 f_sunlit=notgap/(Kb*p.leaf_area_index);
 sunlitlai=p.leaf_area_index*f_sunlit;
 shadedlai=p.leaf_area_index-sunlitlai;
 rad=compute_absorbed_shortwave_rad(p.direct_par_umol_m2_s,p.diffuse_par_umol_m2_s,f_sunlit,0.8,P.par_refl,P.par_omega,costheta,0,zenith,p.leaf_area_index,gaps,P);
 apar_shaded=rad.diffuse/p.leaf_area_index;
 apar_sunlit=rad.direct*Kb+apar_shaded;
 [vc,vcs,vcd,vs,vd,psnp]=compute_psn_parameters1(1,1,0,1,p.young_leaf_fraction,p.leaf_area_index,sunlitlai,shadedlai,p.air_temperature_c,Kb,P);
 psnp.m=p.stomatal_slope;psnp.ppe=p.electron_photon_ratio;
 [asun,avsun,ajsun]=compute_bwb_farq_psn1(psnp,apar_sunlit,p.air_temperature_c,p.vapour_pressure_deficit_pa,1,1,p.co2_ppm,P);
 [ashade,avshade,ajshade]=compute_bwb_farq_psn1(psnp,apar_shaded,p.air_temperature_c,p.vapour_pressure_deficit_pa,1,0,p.co2_ppm,P);
 results(k).name=cases(k).name;
 results(k).net_assimilation_umol_m2_s=asun*sunlitlai+ashade*shadedlai;
 results(k).sunlit_assimilation_umol_m2_s=asun;
 results(k).shaded_assimilation_umol_m2_s=ashade;
 results(k).sunlit_leaf_area_index=sunlitlai;
 results(k).shaded_leaf_area_index=shadedlai;
 results(k).sunlit_absorbed_par=apar_sunlit;
 results(k).shaded_absorbed_par=apar_shaded;
 results(k).between_crown_openness=gaps.Kg_openness;
 results(k).within_crown_openness=gaps.Kz_openness;
 results(k).between_crown_gaps=gaps.Kg_gaps;
 results(k).within_crown_gaps=gaps.Kz_gaps;
 results(k).floor_absorbed_par=rad.floor;
end
out=struct('runtime',version,'mode','provided Main4PSN mvp=1 flagSeason=0 pointwise photosynthesis path; original functions unchanged','cases',results);
fid=fopen(fullfile(root,'benchmarks','reference','gort-gpp','expected.json'),'w');
fprintf(fid,'%s\n',jsonencode(out,PrettyPrint=true));fclose(fid);
disp('GORT-GPP original references generated.');
