/* Auto-generated C header for osm_registry FFI. DO NOT EDIT. */
#ifndef OSM_REGISTRY_FFI_H
#define OSM_REGISTRY_FFI_H

#ifdef __cplusplus
extern "C" {
#endif

/* register instruction */
char* osm_registry_register(const char* args_json);

/* register_batch instruction */
char* osm_registry_register_batch(const char* args_json);

/* unhost instruction */
char* osm_registry_unhost(const char* args_json);

/* fetch region_account account state */
char* osm_registry_fetch_region_account(const char* args_json);

/* wallet helpers */
char* osm_registry_check_connection(const char* args_json);
char* osm_registry_inspect_account(const char* args_json);
char* osm_registry_list_accounts(const char* args_json);
char* osm_registry_create_account(const char* args_json);

void osm_registry_free_string(char* s);
char* osm_registry_version(void);
char* osm_registry_program_id(void);
char* osm_registry_decode_account(const char* args_json);

#ifdef __cplusplus
}
#endif

#endif /* OSM_REGISTRY_FFI_H */
