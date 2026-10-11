export interface RepositoryView {
  root: string;
  schemaVersion: number | null;
  fingerprint: FingerprintView;
  hostCount: number;
  userCount: number;
  profileCount: number;
  applicationCount: number;
  capabilityCount: number;
  optionCount: number;
  activeRequestCount: number;
  errorCount: number;
  warningCount: number;
}

export interface FingerprintView {
  gitHead: string | null;
  workingTreeHash: string | null;
  flakeLockHash: string | null;
  schemaVersion: number | null;
  indexSchemaVersion: number;
}

export interface HostView {
  name: string;
  system: string;
  class: string;
  roles: string[];
  desktop: string;
  shell: string;
  stateVersion: string;
  hardware: HardwareView;
}

export interface HardwareView {
  cpu: string;
  gpu: string;
  hasBattery: boolean;
  hasBluetooth: boolean;
  hasTouchpad: boolean;
  hasPrinter: boolean;
}

export interface UserView {
  username: string;
  fullName: string;
  email: string;
  isMain: boolean;
}

export interface CapabilityView {
  id: { scope: string; domain: string; name: string };
  requestPath: string;
  modulePath: string;
  requesters: string[];
  active: boolean;
  ownedOptions: string[];
  ownershipStatus: string | null;
}

export interface ApplicationView {
  id: { scope: string; category: string; name: string };
  modulePath: string;
  optionPath: string;
  requesterName: string;
  requestedCapabilities: string[];
}

export interface ProfileView {
  id: { scope: string; name: string };
  sourceFile: string;
  requesterName: string;
  requestedCapabilities: string[];
  defaultOptions: string[];
}

export interface OptionView {
  path: string;
  schema: OptionSchemaView | null;
  effective: unknown;
  provenance: ProvenanceView | null;
  owner: OwnerView | null;
}

export interface OptionSchemaView {
  path: string;
  scope: string;
  nixType: string;
  default: unknown;
  description: string | null;
  example: unknown;
  internal: boolean;
  readOnly: boolean;
}

export interface ProvenanceView {
  optionPath: string;
  effectiveValue: unknown;
  definitions: DefinitionView[];
  winning: DefinitionView | null;
  overridden: DefinitionView[];
  contributions: ContributionView[];
}

export interface DefinitionView {
  file: string;
  line: number;
  column: number;
  endLine: number;
  value: unknown;
  layer: string;
  priority: number;
  isDefault: boolean;
  scope: string;
  host: string | null;
  user: string | null;
  profile: string | null;
  condition: ConditionView | null;
  definitionType: string;
}

export interface ConditionView {
  expression: string;
  evaluatedValue: unknown;
  satisfied: boolean | null;
}

export interface ContributionView {
  value: unknown;
  file: string | null;
  line: number | null;
  layer: string | null;
  priority: number | null;
  definitionType: string | null;
  host: string | null;
  user: string | null;
  profile: string | null;
}

export interface OwnerView {
  optionPath: string;
  ownerModule: string;
  ownerKind: string;
  status: string;
  contributingFiles: string[];
}

export interface GraphView {
  nodes: NodeView[];
  edges: EdgeView[];
}

export interface NodeView {
  id: string;
  kind: string;
  label: string;
}

export interface EdgeView {
  from: string;
  to: string;
  kind: string;
}

export interface DoctorView {
  findings: FindingView[];
  repairs: RepairView[];
  tier: string;
  errorCount: number;
  warningCount: number;
}

export interface FindingView {
  severity: string;
  message: string;
  path: string | null;
  file: string | null;
}

export interface RepairView {
  action: string;
  description: string;
  confidence: number;
}

export interface SearchHit {
  kind: string;
  name: string;
  detail: string;
}

export interface WhyChain {
  via: string;
  steps: string[];
}
