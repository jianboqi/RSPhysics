"""Independent published-equation reference, using SciPy Brent roots.

Requires NumPy/SciPy only for generating the committed fixture. The package
and offline tests do not depend on SciPy. No model source code is imported.
"""
from pathlib import Path
import json
import math
from scipy.optimize import brentq

DEFAULTS = dict(
    leaf_temperature_c=25., co2_surface_umol_mol=400., intercellular_co2_umol_mol=280.,
    pressure_pa=101325., oxygen_mmol_mol=210., vcmax25_umol_m2_s=80.,
    jmax25_umol_m2_s=160., respiration25_umol_m2_s=1.2,
    electron_quantum_yield=.24, light_response_curvature=.7, tpu_umol_m2_s=12.,
    vapour_pressure_deficit_kpa=1., minimum_stomatal_conductance_water_mol_m2_s=.01,
    ball_berry_slope=9., leuning_slope_co2=6., leuning_d0_kpa=1.5,
    medlyn_slope_kpa_sqrt=4., medlyn_vpd_floor_kpa=.05,
    vcmax_activation_j_mol=65330., jmax_activation_j_mol=43540.,
    respiration_activation_j_mol=46390., vcmax_deactivation_j_mol=200000.,
    jmax_deactivation_j_mol=200000., vcmax_entropy_j_mol_k=650.,
    jmax_entropy_j_mol_k=650., gas_exchange_mode='stomatal', stomatal_model='medlyn',
    temperature_response='arrhenius', tpu_limitation='off',
    leuning_compensation='net_compensation',
)
R=8.31446261815324


def evaluate(model, overrides):
    p=DEFAULTS|overrides
    canopy=model.endswith('big-leaf')
    area=p.get('leaf_area_index',3.) if canopy else 1.
    kn=p.get('nitrogen_extinction_coefficient',.3)
    scale=(-math.expm1(-kn*area)/kn if kn else area) if canopy else 1.
    light=p.get('absorbed_par_ground_umol_m2_s' if canopy else 'absorbed_par_umol_m2_s',1200.)
    t=p['leaf_temperature_c']+273.15
    def arrh(e):return math.exp(e*(t-298.15)/(R*t*298.15))
    def factor(prefix):
        value=arrh(p[prefix+'_activation_j_mol'])
        if p['temperature_response']=='peaked_arrhenius':
            hd=p[prefix+'_deactivation_j_mol'];s=p[prefix+'_entropy_j_mol_k']
            value*=(1+math.exp((s-hd/298.15)/R))/(1+math.exp((s-hd/t)/R))
        return value
    vc=p['vcmax25_umol_m2_s']*scale*factor('vcmax')
    jm=p['jmax25_umol_m2_s']*scale*factor('jmax')
    rd=p['respiration25_umol_m2_s']*scale*arrh(p['respiration_activation_j_mol'])
    kc=404.9*arrh(79430.)*101325./p['pressure_pa']
    ko=278.4*arrh(36380.)*101325./p['pressure_pa']
    gamma=42.75*arrh(37830.)*p['oxygen_mmol_mol']/210.
    km=kc*(1+p['oxygen_mmol_mol']/ko)
    tp=3*p['tpu_umol_m2_s']*scale-rd
    enabled=p['tpu_limitation']=='simple'
    theta=p['light_response_curvature']
    def electron(q):
        x=p['electron_quantum_yield']*q
        if not x or not jm:return 0.
        if not theta:return x*jm/(x+jm)
        # Ordinary quadratic reference; Rust uses its rationalized form.
        return ((x+jm)-math.sqrt(max(0.,(x+jm)**2-4*theta*x*jm)))/(2*theta)
    j=electron(light)
    def branches(c, electrons=j):
        a=vc*(c-gamma)/(c+km)-rd
        b=electrons/4*(c-gamma)/(c+2*gamma)-rd if c+2*gamma else electrons/4-rd
        net=max(a,b) if c<gamma else min(a,b)
        if enabled:net=min(net,tp)
        return net,a,b
    g0=p['minimum_stomatal_conductance_water_mol_m2_s']*area/1.6
    cs=p['co2_surface_umol_mol'];vpd=p['vapour_pressure_deficit_kpa']
    stomatal=p['gas_exchange_mode']=='stomatal'
    def conductance_co2(a):
        if a<=0:return g0
        if p['stomatal_model']=='medlyn':
            slope=(1+p['medlyn_slope_kpa_sqrt']/math.sqrt(max(vpd,p['medlyn_vpd_floor_kpa'])))/cs
        elif p['stomatal_model']=='ball_berry':
            es=.6108*math.exp(17.27*p['leaf_temperature_c']/(p['leaf_temperature_c']+237.3))
            slope=p['ball_berry_slope']*(1-vpd/es)/(1.6*cs)
        else:
            comp=gamma
            if p['leuning_compensation']=='net_compensation':
                if vc<=rd or j<=4*rd or enabled and tp<=0:return g0
                comp=max((vc*gamma+rd*km)/(vc-rd),gamma*(j+8*rd)/(j-4*rd))
            if cs<=comp:return g0
            slope=p['leuning_slope_co2']/((cs-comp)*(1+vpd/p['leuning_d0_kpa']))
        return g0+slope*a
    if not area:
        ci=None;a=ac=aj=tp=0.;gsc=0. if stomatal else None;residual=0. if stomatal else None;code=4
    else:
        if not stomatal:ci=p['intercellular_co2_umol_mol']
        elif not g0:
            ci=cs-1/conductance_co2(1.)
            assert ci>=0 and branches(ci)[0]>0
        else:
            ci=brentq(lambda c:branches(c)[0]-conductance_co2(branches(c)[0])*(cs-c),0.,5000.,xtol=1e-11,rtol=1e-14)
        a,ac,aj=branches(ci)
        mask=[abs(v-a)<=1e-10*max(1.,abs(a)) for v in ([ac,aj,tp] if enabled else [ac,aj])]
        code=mask.index(True) if sum(mask)==1 else 3
        gsc=conductance_co2(a) if stomatal else None
        residual=a-gsc*(cs-ci) if stomatal else None
    point=dict(net_assimilation_umol_m2_s=a,assimilation_before_day_respiration_umol_m2_s=a+rd,
        rubisco_limited_assimilation_umol_m2_s=ac,electron_limited_assimilation_umol_m2_s=aj,
        tpu_limited_assimilation_umol_m2_s=tp,day_respiration_umol_m2_s=rd,
        vcmax_umol_m2_s=vc,jmax_umol_m2_s=jm,electron_transport_umol_m2_s=j if area else 0.,
        kc_umol_mol=kc,ko_mmol_mol=ko,gamma_star_umol_mol=gamma,michaelis_menten_co2_umol_mol=km,
        intercellular_co2_umol_mol=ci,stomatal_conductance_co2_mol_m2_s=gsc,
        stomatal_conductance_water_mol_m2_s=gsc*1.6 if gsc is not None else None,
        transpiration_mmol_m2_s=gsc*1.6*vpd*1e6/p['pressure_pa'] if gsc is not None else None,
        co2_flux_residual_umol_m2_s=residual,limitation_code=code,
        gas_exchange_computed=stomatal and area>0,intercellular_co2_defined=area>0,tpu_limitation_enabled=enabled,
        capacity_area_factor=scale,leaf_area_factor=area)
    curves=dict(ci_axis_umol_mol=[i*25. for i in range(61)],light_axis_umol_m2_s=[i*80. for i in range(51)])
    for index,name in enumerate(['ci_net_assimilation','ci_rubisco_limited_assimilation','ci_electron_limited_assimilation']):
        curves[name]=[branches(c)[index] if area else 0. for c in curves['ci_axis_umol_mol']]
    curves['light_net_assimilation']=[branches(ci,electron(q))[0] if ci is not None else 0. for q in curves['light_axis_umol_m2_s']]
    return point,curves


def main():
    cases=[]
    def add(model,name,**parameters):
        point,curves=evaluate(model,parameters)
        record=dict(model_id=model,name=name,parameters=parameters,expected=point)
        if name=='default':record['curves']=curves
        cases.append(record)
    for model in ['ecosystem.fvcb','ecosystem.fvcb-big-leaf']:
        add(model,'default')
        for mode in ['prescribed_ci','medlyn','ball_berry','leuning']:
            for temperature in [10.,25.,40.]:
                for tpu in ['off','simple']:
                    params=dict(leaf_temperature_c=temperature,tpu_limitation=tpu)
                    params.update(gas_exchange_mode='prescribed_ci') if mode=='prescribed_ci' else params.update(stomatal_model=mode)
                    add(model,f'{mode}-{temperature}-{tpu}',**params)
        for mode in ['arrhenius','peaked_arrhenius']:
            add(model,'temperature-'+mode,temperature_response=mode,leaf_temperature_c=35.)
        for stomata in ['medlyn','ball_berry','leuning']:
            add(model,'zero-g0-'+stomata,stomatal_model=stomata,minimum_stomatal_conductance_water_mol_m2_s=0.)
        add(model,'leuning-gamma-star',stomatal_model='leuning',leuning_compensation='gamma_star')
        for key,value in [('pressure_pa',50000.),('pressure_pa',110000.),('oxygen_mmol_mol',0.),('respiration25_umol_m2_s',0.),('light_response_curvature',0.),('light_response_curvature',1.),('vcmax25_umol_m2_s',0.),('jmax25_umol_m2_s',0.),('vapour_pressure_deficit_kpa',0.)]:
            add(model,f'{key}-{value}',**{key:value})
        light='absorbed_par_ground_umol_m2_s' if model.endswith('big-leaf') else 'absorbed_par_umol_m2_s'
        add(model,'dark',**{light:0.})
        add(model,'tpu-zero',tpu_limitation='simple',tpu_umol_m2_s=0.)
    for lai,kn in [(0.,.3),(1.,0.),(3.,0.),(3.,1e-12),(12.,3.)]:
        args=dict(leaf_area_index=lai,nitrogen_extinction_coefficient=kn)
        if not lai:args['absorbed_par_ground_umol_m2_s']=0.
        add('ecosystem.fvcb-big-leaf',f'area-{lai}-{kn}',**args)
    for key,value in [('absorbed_par_umol_m2_s',0.),('vcmax25_umol_m2_s',0.),('jmax25_umol_m2_s',0.)]:
        add('ecosystem.fvcb','low-ci-limit-'+key,gas_exchange_mode='prescribed_ci',intercellular_co2_umol_mol=0.,**{key:value})
    destination=Path(__file__).with_name('reference.json')
    destination.write_text(json.dumps(dict(suite='fvcb-independent-equations-v1',cases=cases),indent=2,allow_nan=False)+'\n',encoding='utf8',newline='\n')
    print('Generated',len(cases),'independent reference cases')


if __name__=='__main__':main()
