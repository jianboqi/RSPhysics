# RPV reference benchmark

420 selected BRFs are from the JRC RAMI rpv2 benchmark archive, at the
published six-decimal precision. The original Fortran uses single-precision REAL variables. Tolerance is1.1e-6 absolute for these published outputs; an independent80-digit equation fixture additionally verifies double precision at2e-12.
All six parameter sets and both solar zeniths are preserved. Additional
tests verify reciprocity, three/four-parameter equivalence and Lambertian
limits; all angular arrays are independently evaluated in Python tests.
Source: https://rami-benchmark.jrc.ec.europa.eu/_www/definition.php.
Archive: https://rami-benchmark.jrc.ec.europa.eu/_www/comm/images/rpv2-benchmark.zip.
