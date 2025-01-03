import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from '../../environments/environment';
import { buildMocked } from './mock';
import {
  AdminPanelOverview,
  AdminPanelUserInfo,
  CreateUserParameters,
  LoginResponse,
  UserData,
} from '../types/model';

const MOCKED = false;
const API_URL = environment.apiHost + '/api';

@Injectable({
  providedIn: 'root',
})
export class ApiService {
  constructor(private httpClient: HttpClient) {}

  login(username: string, password: string): Observable<LoginResponse> {
    return MOCKED
      ? buildMocked({
          token: 'token',
          tokenType: 'Bearer',
        })
      : this.httpClient.post<LoginResponse>(API_URL + '/auth/login', {
          username,
          password,
        });
  }

  getApiStringExample(): Observable<string> {
    return MOCKED
      ? // buildMockedError():
        buildMocked('IT WORKS!')
      : this.httpClient.get<string>(API_URL + '/api-string');
  }

  getUserData(): Observable<UserData> {
    return MOCKED
      ? buildMocked({
          id: 'my-id',
          username: 'my-username',
          name: 'my-name',
          surname: 'my-surname',
          email: 'my-email',
          platformAdmin: true,
          active: true,
        })
      : this.httpClient.get<UserData>(API_URL + '/auth/user');
  }

  getAdminPanelOverview(): Observable<AdminPanelOverview> {
    return MOCKED
      ? buildMocked({
          totalUsers: 10,
          totalAdmins: 2,
          totalActiveUsers: 8,
          totalInactiveUsers: 2,
          totalCompanies: 5,
        })
      : this.httpClient.get<AdminPanelOverview>(API_URL + '/admin/overview');
  }

  getAdminUsersInfo(): Observable<AdminPanelUserInfo[]> {
    return MOCKED
      ? buildMocked(
          [...Array(50).keys()].map((i) => ({
            id: `my-id-${i}`,
            username: `my-username-${i}`,
            name: `my-name-${i}`,
            surname: `my-surname-${i}`,
            email: `my-email-${i}`,
            platformAdmin: i % 2 === 0,
            active: i % 3 === 0,
            totalCompanies: i,
          }))
        )
      : this.httpClient.get<AdminPanelUserInfo[]>(API_URL + '/admin/user');
  }

  setPlatformAdminUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/admin/user/${userId}/platform-admin`,
          {}
        );
  }

  unsetPlatformAdminUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/admin/user/${userId}/platform-admin`,
          {}
        );
  }

  activateUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.post<void>(
          API_URL + `/admin/user/${userId}/activate`,
          {}
        );
  }

  deactivateUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(
          API_URL + `/admin/user/${userId}/activate`,
          {}
        );
  }

  deleteUser(userId: string): Observable<void> {
    return MOCKED
      ? buildMocked()
      : this.httpClient.delete<void>(API_URL + `/admin/user/${userId}`, {});
  }

  createUser(user: CreateUserParameters): Observable<string> {
    return MOCKED
      ? buildMocked('new-user-id')
      : this.httpClient.post<string>(API_URL + '/admin/user', user);
  }
}
