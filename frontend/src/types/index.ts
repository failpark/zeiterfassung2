// User types
export interface User {
  id: number;
  username: string;
  firstname: string;
  lastname: string;
  email: string;
  hash: string;
  sys_role: string;
  created_at: string;
  updated_at: string;
}

export interface CreateUser {
  username: string;
  firstname: string;
  lastname: string;
  email: string;
  password: string;
  sys_role: string;
}

export interface UpdateUser {
  username?: string;
  firstname?: string;
  lastname?: string;
  email?: string;
  password?: string;
  sys_role?: string;
}

// Client types
export interface Client {
  id: number;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface CreateClient {
  name: string;
}

export interface UpdateClient {
  name?: string;
}

// Project types
export interface Project {
  id: number;
  client_id: number;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface CreateProject {
  client_id: number;
  name: string;
}

export interface UpdateProject {
  name?: string;
}

// Activity types
export interface Activity {
  id: number;
  token?: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface CreateActivity {
  token?: string;
  name: string;
}

export interface UpdateActivity {
  token?: string;
  name?: string;
}

// Tracking types
export interface Tracking {
  id: number;
  client_id: number;
  user_id: number;
  project_id: number;
  date: string; // ISO date string
  begin: string; // ISO time string
  end: string; // ISO time string
  pause?: string; // ISO time string
  performed: number;
  billed: number;
  description?: string;
  created_at: string;
  updated_at: string;
  activities?: number[]; // IDs of associated activities
}

export interface CreateTracking {
  client_id: number;
  user_id: number;
  project_id: number;
  date: string; // ISO date string
  begin: string; // ISO time string
  end: string; // ISO time string
  pause?: string; // ISO time string
  performed: number;
  billed: number;
  description?: string;
  activities: number[]; // IDs of associated activities
}

export interface UpdateTracking {
  client_id?: number;
  user_id?: number;
  project_id?: number;
  date?: string;
  begin?: string;
  end?: string;
  pause?: string | null;
  performed?: number;
  billed?: number;
  description?: string;
  activities?: number[];
}

// Auth types
export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  token: string;
}

// Pagination types
export interface PaginationResult<T> {
  items: T[];
  total_items: number;
  page: number;
  page_size: number;
  num_pages: number;
}