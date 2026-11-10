"""Compare stomatal formulations and run leaf/canopy FvCB calculations locally."""
import argparse
from pathlib import Path
import numpy as np
import rsphysics as rsp


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plot',action='store_true')
    parser.add_argument('--output-dir',type=Path,default=Path('rsphysics-fvcb-output'))
    args=parser.parse_args();args.output_dir.mkdir(parents=True,exist_ok=True)
    for model in ['medlyn','ball_berry','leuning']:
        leaf=rsp.ecosystem.fvcb(stomatal_model=model)['result']
        print(model,'A:',leaf['net_assimilation_umol_m2_s'],'Ci:',leaf['intercellular_co2_umol_mol'],'gsw:',leaf['stomatal_conductance_water_mol_m2_s'])
    canopy=rsp.ecosystem.fvcb_big_leaf(leaf_area_index=3.,absorbed_par_ground_umol_m2_s=1200.)['result']
    print('Single-big-leaf net assimilation per ground area:',canopy['net_assimilation_umol_m2_s'])
    table=rsp.parameter_grid('ecosystem.fvcb',{'intercellular_co2_umol_mol':np.linspace(0.,1500.,61)})
    batch=rsp.ecosystem.fvcb_batch(table,gas_exchange_mode='prescribed_ci',shared_axes=True)
    ci=table[:,rsp.ecosystem.FVCB_PARAMETERS.index('intercellular_co2_umol_mol')]
    a=batch['result']['net_assimilation_umol_m2_s']
    np.savetxt(args.output_dir/'a-ci.csv',np.column_stack((ci,a)),delimiter=',',header='ci_umol_mol,net_assimilation_umol_m2_leaf_s',comments='')
    if args.plot:
        import matplotlib
        matplotlib.use('Agg')
        import matplotlib.pyplot as plt
        fig,ax=plt.subplots();ax.plot(ci,a)
        ax.set(xlabel='Intercellular CO2 (µmol/mol)',ylabel='Net leaf assimilation (µmol CO2 m⁻² s⁻¹)')
        ax.axhline(0,color='grey',lw=.6);fig.tight_layout()
        fig.savefig(args.output_dir/'a-ci.png',dpi=160);plt.close(fig)


if __name__=='__main__':main()
