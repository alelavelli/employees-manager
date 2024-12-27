import { Injectable } from '@angular/core';

const STORAGE_KEY = 'storage-key-jwt';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  jwt: string | null = null;

  constructor() {
    try {
      const storageJwt = localStorage.getItem(STORAGE_KEY);
      this.jwt = storageJwt !== null ? storageJwt : null;
    } catch (error) {
      this.jwt = null;
    }
  }

  setJwtToken(jwt: string) {
    if (jwt !== '') {
      this.jwt = jwt;
      localStorage.setItem(STORAGE_KEY, jwt);
    }
  }

  clear() {
    this.jwt = null;
    localStorage.removeItem(STORAGE_KEY);
  }

  getJwtToken(): string | null {
    return this.jwt;
  }

  isAuthenticated() {
    return this.jwt !== null;
  }
}
