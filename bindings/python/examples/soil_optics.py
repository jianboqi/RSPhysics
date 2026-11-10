"""Independent soil/surface forward models; optional comparison figure."""
import argparse
from pathlib import Path
import numpy as np
import rsphysics as rsp

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plot',action='store_true')
    parser.add_argument('--output-dir',type=Path,default=Path('target/soil-optics-example'))
    args=parser.parse_args();args.output_dir.mkdir(parents=True,exist_ok=True)
    results={
        'gsv':rsp.soil.gsv()['result'],
        'bsm':rsp.soil.bsm()['result'],
        'rpv':rsp.surface.rpv()['result'],
        'soilspect':rsp.soil.soilspect(quadrature='refined')['result'],
    }
    for name,out in results.items():
        if name in ['gsv','bsm']:
            keys=['wavelengths_nm','dry_reflectance','reflectance' if name=='gsv' else 'wet_reflectance']
        else:keys=['principal_view_zenith_deg','principal_brf']
        np.savetxt(args.output_dir/f'{name}.csv',np.column_stack([out[k] for k in keys]),delimiter=',',header=','.join(keys),comments='')
        print(f'{name}: {len(out[keys[0]])} output samples')
    # Each row is an independent model run, without coupling.
    grid=rsp.parameter_grid('soil.bsm',{'soil_moisture_percent':[0,20,50]})
    batch=rsp.soil.bsm_batch(grid)
    assert batch['result']['wet_reflectance'].shape==(3,2001)
    if args.plot:
        import matplotlib
        matplotlib.use('Agg')
        import matplotlib.pyplot as plt
        fig,axes=plt.subplots(2,2,figsize=(11,7),layout='constrained')
        for name,ax in zip(results,axes.flat):
            out=results[name]
            if name in ['gsv','bsm']:
                ax.plot(out['wavelengths_nm'],out['dry_reflectance'],label='Dry',color='#ac7d3b')
                ax.plot(out['wavelengths_nm'],out['reflectance' if name=='gsv' else 'wet_reflectance'],label='Wet',color='#247456')
                ax.set(xlabel='Wavelength (nm)',ylabel='Reflectance');ax.legend()
            else:
                ax.plot(out['principal_view_zenith_deg'],out['principal_brf'],color='#247456',label='BRF')
                ax.set(xlabel='Signed view zenith (deg; positive backward)',ylabel='BRF');ax.legend()
            ax.set_title(name.upper());ax.grid(alpha=.2)
        fig.savefig(args.output_dir/'soil-optics.png',dpi=180)
    print('Outputs:',args.output_dir.resolve())

if __name__=='__main__':main()
