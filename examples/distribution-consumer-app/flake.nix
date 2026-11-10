{
  description = "Template: a Basecamp app that shows OSM map data through the pramadani_osm_distribution module (LP-0018)";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
    pramadani_osm_distribution.url = "github:pramadanif/logos-osm-distribution?dir=basecamp/osm-distribution";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosQmlModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
