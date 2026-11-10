cmake_minimum_required(VERSION 3.14)
project(PramadaniOsmAppPlugin LANGUAGES CXX)

if(DEFINED ENV{LOGOS_MODULE_BUILDER_ROOT})
    include($ENV{LOGOS_MODULE_BUILDER_ROOT}/cmake/LogosModule.cmake)
else()
    message(FATAL_ERROR "LogosModule.cmake not found. Set LOGOS_MODULE_BUILDER_ROOT.")
endif()

# Read NAME from metadata.json: the builder builds the backend from a copy named
# pramadani_osm_app_backend.
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/metadata.json" METADATA_JSON)
string(JSON MODULE_NAME GET ${METADATA_JSON} name)

logos_module(
    NAME ${MODULE_NAME}
    REP_FILE src/pramadani_osm_app.rep
    SOURCES
        src/pramadani_osm_app_backend.h
        src/pramadani_osm_app_backend.cpp
    INCLUDE_DIRS
        src
)
