export type UserRole = "admin" | "manager" | "investigator" | "viewer";

export interface UserPublic {
  id: string;
  email: string;
  username: string;
  role: UserRole | string;
  name: string;
  avatar?: string | null;
  is_active: boolean;
  last_login?: string | null;
  created_at: string;
}

export interface Case {
  id: string;
  case_number: string;
  subject: string;
  investigation_type: string;
  client: string;
  clients_client?: string | null;
  client_contact?: string | null;
  client_file?: string | null;
  investigator_id?: string | null;
  assigned_to_id?: string | null;
  stage: string;
  priority: string;
  is_rush: boolean;
  is_rework: boolean;
  is_death: boolean;
  opened_date: string;
  completed_date?: string | null;
  case_notes?: string | null;
  additional_info?: string | null;
  created_by: string;
  stage_changed_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface DashboardStats {
  total_cases: number;
  active_cases: number;
  completed_cases: number;
  rush_cases: number;
  stalled_cases: number;
  by_stage: { stage: string; count: number }[];
}

export interface CreateCaseInput {
  case_number?: string;
  subject: string;
  investigation_type: string;
  client: string;
  clients_client?: string;
  client_contact?: string;
  client_file?: string;
  investigator_id?: string;
  assigned_to_id?: string;
  stage?: string;
  priority?: string;
  is_rush?: boolean;
  is_rework?: boolean;
  opened_date?: string;
  completed_date?: string;
  case_notes?: string;
  additional_info?: string;
  case_status?: string;
}
