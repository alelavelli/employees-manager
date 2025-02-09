import { Injectable } from '@angular/core';
import { UserData } from '../types/model';
import { ApiService } from './api.service';
import { BehaviorSubject, map, Observable, of } from 'rxjs';

const STORAGE_KEY = 'storage-key-jwt';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  jwt: string | null = null;
  userData: UserData | null = null;

  private userDataSubject = new BehaviorSubject<UserData | null>(null);
  userData$: Observable<UserData> | null = null;

  constructor(private apiService: ApiService) {
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
    this.userData = null;
    this.userDataSubject = new BehaviorSubject<UserData | null>(null);
    this.userData$ = null;
    localStorage.removeItem(STORAGE_KEY);
  }

  getJwtToken(): string | null {
    return this.jwt;
  }

  fetchUserData(): Observable<UserData> {
    if (this.userData !== null) {
      return of(this.userData);
    } else if (this.userData$ !== null) {
      return this.userData$;
    } else {
      return this.setUserData();
    }
  }

  setUserData(): Observable<UserData> {
    this.userData$ = this.apiService.getUserData().pipe(
      map((userData) => {
        this.userData = userData;
        this.userDataSubject.next(userData);

        return userData;
      })
    );

    return this.userData$;
  }

  getUserData(): UserData | null {
    return this.userData;
  }

  isAuthenticated() {
    return this.jwt !== null;
  }

  isPlatformAdmin(): Observable<boolean> {
    if (this.userData === null) {
      return this.fetchUserData().pipe(
        map((userData) => {
          return userData.platformAdmin;
        })
      );
    } else {
      return of(this.userData.platformAdmin);
    }
  }
}
