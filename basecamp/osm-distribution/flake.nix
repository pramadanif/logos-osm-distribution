{
  description = "pramadani_osm_distribution: LP-0018 OSM distribution core module for Logos Basecamp";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
    # The LEZ wallet module; signs registry writes. Optional at runtime: reads work without it.
    lez_core.url = "github:logos-blockchain/logos-execution-zone-module";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
