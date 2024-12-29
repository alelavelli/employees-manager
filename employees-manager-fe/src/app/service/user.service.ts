import { Injectable } from '@angular/core';
import { UserData } from '../types/model';
import { ApiService } from './api.service';

const STORAGE_KEY = 'storage-key-jwt';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  jwt: string | null = null;
  userData: UserData | null = null;

  constructor(private apiService: ApiService) {
    try {
      const storageJwt = localStorage.getItem(STORAGE_KEY);
      this.jwt = storageJwt !== null ? storageJwt : null;
      this.setUserData();
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
    this.userData = null;
    localStorage.removeItem(STORAGE_KEY);
  }

  getJwtToken(): string | null {
    return this.jwt;
  }

  setUserData() {
    this.apiService.getUserData().subscribe({
      next: (data) => {
        this.userData = data;
      },
    });
  }

  isAuthenticated() {
    return this.jwt !== null;
  }

  isPlatformAdmin() {
    if (this.userData === null) {
      return false;
    } else {
      return this.userData.platformAdmin;
    }
  }
}
