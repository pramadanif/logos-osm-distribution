/* C smoke test for osm_ffi, built with -fsanitize=address,undefined in CI (H37). */
#include <stdio.h>
#include <string.h>
#include "osm_ffi.h"

static int check(const char *what, char *s, const char *needle) {
    int ok = s && strstr(s, needle) != NULL;
    printf("%s %s\n", ok ? "PASS" : "FAIL", what);
    osm_free(s);
    return ok ? 0 : 1;
}

int main(void) {
    int fails = 0;
    fails += check("version", osm_version(), "\"api\":1");
    fails += check("discover offline", osm_discover("{\"geofabrik\":false}"), "\"ok\":true");
    fails += check("bad json", osm_discover("{"), "\"S160\"");
    fails += check("null", osm_discover(NULL), "null pointer");
    fails += check("unreachable rpc", osm_resolve("{\"rpc\":\"http://127.0.0.1:1\",\"program\":\"G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ\",\"region\":\"kenya\"}"), "\"S100\"");
    osm_free(NULL);
    printf("SUMMARY fails=%d\n", fails);
    return fails;
}
