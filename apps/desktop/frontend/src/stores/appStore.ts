import { create } from "zustand";

interface AppState {
  repoPath: string;
  selectedHost: string | null;
  selectedUser: string | null;
  setRepoPath: (path: string) => void;
  setSelectedHost: (host: string | null) => void;
  setSelectedUser: (user: string | null) => void;
}

export const useAppStore = create<AppState>((set) => ({
  repoPath: ".",
  selectedHost: null,
  selectedUser: null,
  setRepoPath: (repoPath) => set({ repoPath }),
  setSelectedHost: (selectedHost) => set({ selectedHost }),
  setSelectedUser: (selectedUser) => set({ selectedUser }),
}));
