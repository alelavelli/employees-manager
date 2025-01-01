export interface LoginResponse {
  token: string;
}

export interface UserData {
  id: string;
  username: string;
  email: string;
  name: string;
  surname: string;
  platformAdmin: boolean;
  active: boolean;
}

export interface AdminPanelOverview {
  totalUsers: number;
  totalAdmins: number;
  totalActiveUsers: number;
  totalInactiveUsers: number;
  totalCompanies: number;
}

export interface AdminPanelUserInfo {
  id: string;
  username: string;
  email: string;
  name: string;
  surname: string;
  platformAdmin: boolean;
  active: boolean;
  totalCompanies: number;
}

export interface CreateUserParameters {
  username: string;
  password: string;
  name: string;
  surname: string;
  email: string;
}
