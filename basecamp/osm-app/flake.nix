{
  description = "pramadani_osm_app: LP-0018 OSM distribution app for Logos Basecamp (QML view + C++ backend)";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
    # The distribution core module this app drives; must match metadata.json "dependencies".
    pramadani_osm_distribution.url = "github:pramadanif/logos-osm-distribution?dir=basecamp/osm-distribution";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosQmlModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
