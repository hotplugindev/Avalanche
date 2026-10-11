import { BrowserRouter, Route, Routes } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Layout } from "@/components/layout/Layout";
import { Dashboard } from "@/pages/Dashboard";
import { Hosts } from "@/pages/Hosts";
import { Users } from "@/pages/Users";
import { Profiles } from "@/pages/Profiles";
import { Applications } from "@/pages/Applications";
import { Capabilities } from "@/pages/Capabilities";
import { Graph } from "@/pages/Graph";
import { Provenance } from "@/pages/Provenance";
import { Doctor } from "@/pages/Doctor";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 30_000,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<Dashboard />} />
            <Route path="hosts" element={<Hosts />} />
            <Route path="users" element={<Users />} />
            <Route path="profiles" element={<Profiles />} />
            <Route path="applications" element={<Applications />} />
            <Route path="capabilities" element={<Capabilities />} />
            <Route path="graph" element={<Graph />} />
            <Route path="provenance" element={<Provenance />} />
            <Route path="doctor" element={<Doctor />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
