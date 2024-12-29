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
