{
  description = "Template: a Logos core module that reads OSM map data through pramadani_osm_sdk (LP-0018)";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
    # Read-only OSM registry SDK; must match metadata.json "dependencies".
    pramadani_osm_sdk.url = "github:pramadanif/logos-osm-distribution?dir=basecamp/osm-sdk";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
