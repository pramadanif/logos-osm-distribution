/* C ABI for the LP-0018 OSM registry SDK (consumer API).
 *
 * Every function takes a NUL-terminated UTF-8 JSON object and returns a newly allocated
 * NUL-terminated JSON string that the caller must release with osm_free(). The answer is
 * {"ok":true,"data":...} or {"ok":false,"code":"S1xx","error":"<plain message>"}; codes are
 * listed in docs/error-codes.md. No function panics across the boundary or blocks forever:
 * network calls have timeouts. Call them off the UI thread (H34).
 */
#ifndef OSM_FFI_H
#define OSM_FFI_H
#ifdef __cplusplus
extern "C" {
#endif

/* {"version":"x.y.z","api":1} */
char *osm_version(void);
/* {"geofabrik":true|false,"geofabrik_base"?:url} -> {"source":..., "regions":[...], "missing":[...]} */
char *osm_discover(const char *args_json);
/* {"rpc","program","region"} -> resolved entry or null */
char *osm_resolve(const char *args_json);
/* {"rpc","program","region","storage","dir","geofabrik":bool} -> {"path","source","from","md5","skipped"} */
char *osm_fetch(const char *args_json);
/* {"rpc","program"} -> [{"region","version","hosts"}] for all 72 regions */
char *osm_hosted_status(const char *args_json);
/* {"rpc","program","regions":[...],"geofabrik_base"?} -> [{"region","registered","available","update_available"}] */
char *osm_check_updates(const char *args_json);
void osm_free(char *s);

#ifdef __cplusplus
}
#endif
#endif
