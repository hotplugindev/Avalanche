{
  schemaVersion = 1;

  description = "Avalanche repository contract. The managed Nix repository declares conformance via gb.schemaVersion. Avalanche validates every repository against this contract before indexing.";

  namespaces = {
    "gb.schemaVersion" = {
      scope = "both";
      kind = "meta";
      type = "int";
      description = "Contract version the repository conforms to.";
    };

    "gb.host" = {
      scope = "system";
      kind = "facts";
      description = "Typed host identity and hardware facts. Hosts describe the machine, never implement services.";
      fields = [
        "name"
        "system"
        "class"
        "roles"
        "desktop"
        "shell"
        "stateVersion"
        "hardware.cpu"
        "hardware.gpu"
        "hardware.hasBattery"
        "hardware.hasBluetooth"
        "hardware.hasTouchpad"
        "hardware.hasPrinter"
      ];
    };

    "gb.user" = {
      scope = "both";
      kind = "identity";
      description = "Active user identity exposed to modules.";
      fields = [ "username" "fullName" "email" ];
    };

    "gb.requires.system" = {
      scope = "system";
      kind = "request";
      type = "listOfStr";
      description = "System capability requests. Value is the list of canonical requester strings.";
    };

    "gb.requires.home" = {
      scope = "home";
      kind = "request";
      type = "listOfStr";
      description = "Home capability requests. Value is the list of canonical requester strings.";
    };

    "gb.programs.system" = {
      scope = "system";
      kind = "application";
      description = "Typed system application settings.";
    };

    "gb.home.programs" = {
      scope = "home";
      kind = "application";
      description = "Typed home application settings.";
    };

    "gb.home.desktop" = {
      scope = "home";
      kind = "desktop";
      description = "Structured desktop configuration (Mango, DMS, etc.).";
    };

    "gb.debug" = {
      scope = "system";
      kind = "debug";
      description = "Diagnostic switches. Never affects production configuration semantics.";
    };
  };

  requesterGrammar = {
    description = "Every entry in a gb.requires.* list is a canonical requester string.";
    form = "<kind>.<scope>[.<domain>].<name>";
    kinds = [
      "programs"
      "profiles"
      "capabilities"
      "hosts"
      "users"
    ];
    scopedKinds = [
      "programs"
      "profiles"
      "capabilities"
    ];
    unscopedKinds = [
      "hosts"
      "users"
    ];
    scopes = [
      "system"
      "home"
    ];
    examples = {
      program = "programs.home.ai.codex";
      profile = "profiles.system.gaming";
      capability = "capabilities.system.audio.pipewire";
      host = "hosts.pc";
      user = "users.hotplugin";
    };
    invalidExamples = [
      "profile.system.development"
      "program.home.spotify"
      "host.pc"
    ];
    rule = "Kind prefixes are always plural. Singular forms are contract violations and must be rejected by the doctor.";
  };

  ownership = {
    convention = "explicit-declaration";
    inferredFromFilename = false;
    statuses = [
      "owned"
      "unowned"
      "wrongOwner"
      "ambiguous"
      "external"
    ];
    rule = "One upstream option has exactly one owning capability or program module. Ownership is declared, never inferred from the nearest filename.";
    roles = [
      "owner"
      "requester"
      "implementer"
    ];
  };

  capabilityRegistration = {
    location = "modules/capabilities/<scope>/<domain>/<name>.nix";
    requestOption = "gb.requires.<scope>.<domain>.<name>";
    activation = "nonEmptyRequestList or explicitEnable";
    description = "A capability owns one real domain and activates when its request list is non-empty or it is explicitly enabled.";
  };

  aggregateRegistration = {
    system = "modules/aggregate/nixos.nix";
    home = "modules/aggregate/home.nix";
    rule = "Every module must be imported by an aggregate. No recursive auto-import. A module absent from an aggregate is inert.";
  };

  profileSemantics = {
    location = "modules/profiles/<scope>/<name>.nix";
    expresses = "intent";
    mayRequestCapabilities = true;
    maySetDefaults = true;
    usesMkDefault = true;
    mustNotOwnImplementation = true;
    description = "Profiles describe what a machine is meant to be. They request capabilities and set mkDefault values; they never implement low-level behavior.";
  };

  hostOverrideSemantics = {
    systemLocation = "hosts/<host>/default.nix";
    homeLocation = "hosts/<host>/home/<username>.nix";
    resetMeans = "deleteHostDefinition";
    notWriteFalse = true;
    description = "A host override is a definition at host priority. Reset to profile default deletes the host definition; it never writes false.";
  };

  userScoping = {
    identity = "users/<username>.nix";
    perHost = "hosts/<host>/home/<username>.nix";
    sharedHostHomeDeprecated = true;
    description = "Per-user configuration has an explicit source location. The single shared hosts/<host>/home.nix is deprecated in favor of per-user files.";
  };

  externalModules = {
    origin = "external";
    editable = false;
    configureVia = "localRepositoryLayer";
    description = "Options originating from external flake inputs are tagged external. Avalanche configures the local layer and never edits the dependency.";
  };
}
