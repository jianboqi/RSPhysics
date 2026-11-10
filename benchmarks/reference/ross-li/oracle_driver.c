/* Test-only ABI bridge; includes the unchanged downloaded original C source.
 * Never linked into RSPhysics. _USE_MATH_DEFINES supplies Windows math constants.
 */
#define _USE_MATH_DEFINES
#include "ambralsfor.c"
#ifdef _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif
EXPORT double oracle(double s,double v,double a,double iso,double vol,double geo) {
    geom_t geometry={(float)s,(float)v,(float)a};
    param_t weights={(float)iso,(float)vol,(float)geo};
    return (double)forward(geometry,weights);
}
