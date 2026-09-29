export interface Host {
  name: string;
  system: string;
  class: string;
  roles: string[];
  desktop: string;
  shell: string;
  state_version: string;
}

export interface OptionSchema {
  path: string;
  scope: string;
  nix_type: string;
  default: unknown;
  description: string | null;
  internal: boolean;
  read_only: boolean;
}

export interface Transaction {
  id: string;
  intents: MutationIntent[];
  status: string;
}

export type MutationIntent =
  | { type: "set_option"; path: string; value: unknown; scope: string }
  | { type: "set_profile_default"; path: string; value: unknown; profile: string }
  | { type: "set_host_override"; path: string; value: unknown; host: string }
  | { type: "reset_host_override"; path: string; host: string }
  | { type: "enable_capability"; capability: string; requester: string }
  | { type: "disable_capability"; capability: string; requester: string }
  | { type: "add_request"; capability: string; requester: string }
  | { type: "remove_request"; capability: string; requester: string };
