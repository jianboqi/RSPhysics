# FLUSPECT-CX component

Scientific model: Nastassia Vilfan, Christiaan van der Tol, Wout Verhoef,
Joris Timmermans and colleagues. Adapted from the authors' SCOPE source
fluspect_B_CX.m at revision f4d07b33828f82c572cb1965cf82b1db4579c5b4.
Source and coefficients: https://github.com/Christiaanvandertol/SCOPE

The scientific implementation and optical data retain GPL-3.0 terms.
The original upstream license is retained in the component's
UPSTREAM_LICENSE.txt; a complete GPLv3 text is included as LICENSE.
The Apache-2.0 license for project-original code does not replace these terms.

The optical tables are Optipar2017_ProspectD.mat and
Optipar2021_ProspectPRO_CX.mat, exported to canonical CSV with 17 significant
digits and compressed losslessly. The PRO table supports protein/CBC inputs. The Rust implementation retains the 15 doubling steps,
selected excitation-bin weight, and full 1 nm or original light 5/4 nm grids.
The 641 nm negative calibration residue is set to zero for physical emission;
raw coefficients and original negative matrix rows remain recorded. At zero absorption,
the analytic conservative Stokes limit is used; at zero efficiency, both
fluorescence matrices are explicitly zero rather than absent.

References: Vilfan et al. (2016), DOI 10.1016/j.rse.2016.09.017;
Vilfan et al. (2018), DOI 10.1016/j.rse.2018.04.012.
