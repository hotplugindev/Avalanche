import { useQuery } from "@tanstack/react-query";
import { call } from "@/ipc/client";
import { useAppStore } from "@/stores/appStore";
import type {
  RepositoryView,
  HostView,
  UserView,
  ProfileView,
  ApplicationView,
  CapabilityView,
  GraphView,
  DoctorView,
  ProvenanceView,
  SearchHit,
  WhyChain,
} from "@/types";

function useRepoPath() {
  return useAppStore((s) => s.repoPath);
}

export function useRepository() {
  const repoPath = useRepoPath();
  return useQuery<RepositoryView>({
    queryKey: ["repository", repoPath],
    queryFn: () => call("get_repository"),
  });
}

export function useHosts() {
  const repoPath = useRepoPath();
  return useQuery<HostView[]>({
    queryKey: ["hosts", repoPath],
    queryFn: () => call("get_hosts"),
  });
}

export function useUsers() {
  const repoPath = useRepoPath();
  return useQuery<UserView[]>({
    queryKey: ["users", repoPath],
    queryFn: () => call("get_users"),
  });
}

export function useProfiles() {
  const repoPath = useRepoPath();
  return useQuery<ProfileView[]>({
    queryKey: ["profiles", repoPath],
    queryFn: () => call("get_profiles"),
  });
}

export function useApplications() {
  const repoPath = useRepoPath();
  return useQuery<ApplicationView[]>({
    queryKey: ["applications", repoPath],
    queryFn: () => call("get_applications"),
  });
}

export function useCapabilities() {
  const repoPath = useRepoPath();
  return useQuery<CapabilityView[]>({
    queryKey: ["capabilities", repoPath],
    queryFn: () => call("get_capabilities"),
  });
}

export function useGraph() {
  const repoPath = useRepoPath();
  return useQuery<GraphView>({
    queryKey: ["graph", repoPath],
    queryFn: () => call("get_graph"),
  });
}

export function useDoctor() {
  const repoPath = useRepoPath();
  return useQuery<DoctorView>({
    queryKey: ["doctor", repoPath],
    queryFn: () => call("get_doctor"),
  });
}

export function useProvenance(optionPath: string | undefined) {
  const repoPath = useRepoPath();
  return useQuery<ProvenanceView>({
    queryKey: ["provenance", repoPath, optionPath],
    queryFn: () => call("get_provenance", { optionPath: optionPath! }),
    enabled: !!optionPath,
  });
}

export function useSearch(query: string) {
  const repoPath = useRepoPath();
  return useQuery<SearchHit[]>({
    queryKey: ["search", repoPath, query],
    queryFn: () => call("search", { query }),
    enabled: query.length > 1,
  });
}

export function useWhy(input: string | undefined) {
  const repoPath = useRepoPath();
  return useQuery<WhyChain[]>({
    queryKey: ["why", repoPath, input],
    queryFn: () => call("get_why", { input: input! }),
    enabled: !!input,
  });
}
