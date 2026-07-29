import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth-options";
import type { Case, CreateCaseInput, DashboardStats, UserPublic } from "@/lib/types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://127.0.0.1:8080";

async function authHeader(): Promise<HeadersInit> {
  const session = await getServerSession(authOptions);
  const token = (session as { accessToken?: string } | null)?.accessToken;
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export async function apiClient<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const headers: HeadersInit = {
    "Content-Type": "application/json",
    ...(await authHeader()),
    ...options.headers,
  };

  const res = await fetch(`${API_URL}/api/v1${endpoint}`, {
    ...options,
    headers,
    cache: "no-store",
  });

  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || `API ${res.status}`);
  }
  return res.json();
}

/** Browser-side client using token from session via caller */
export async function browserApi<T>(
  endpoint: string,
  token: string | undefined,
  options: RequestInit = {}
): Promise<T> {
  const res = await fetch(`${API_URL}/api/v1${endpoint}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...options.headers,
    },
    cache: "no-store",
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({}));
    throw new Error(err.message || `API ${res.status}`);
  }
  return res.json();
}

export const casesAPI = {
  list: (qs = "") =>
    apiClient<{ cases: Case[]; total: number }>(`/cases${qs}`),
  get: (id: string) => apiClient<{ case: Case }>(`/cases/${id}`),
  create: (data: CreateCaseInput) =>
    apiClient<{ case: Case }>("/cases", {
      method: "POST",
      body: JSON.stringify(data),
    }),
  updateStage: (id: string, stage: string, note?: string) =>
    apiClient<{ case: Case }>(`/cases/${id}/stage`, {
      method: "PATCH",
      body: JSON.stringify({ stage, note }),
    }),
  clients: () => apiClient<{ clients: string[] }>("/cases/clients"),
  nextId: () => apiClient<{ case_number: string }>("/cases/next-id"),
};

export const dashboardAPI = {
  stats: () => apiClient<{ stats: DashboardStats }>("/dashboard"),
};

export const usersAPI = {
  list: () => apiClient<{ users: UserPublic[] }>("/users"),
  create: (data: {
    email: string;
    username: string;
    password: string;
    name: string;
    role: string;
  }) =>
    apiClient<{ user: UserPublic }>("/users", {
      method: "POST",
      body: JSON.stringify(data),
    }),
};
