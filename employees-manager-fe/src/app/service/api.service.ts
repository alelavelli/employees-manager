import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from '../../environments/environment';
import { MOCK_LOGIN_RESPONSE, buildMocked } from './mock';
import { UserData } from '../types/model';

const MOCKED = true;
const API_URL = environment.apiHost + '/api';

@Injectable({
  providedIn: 'root',
})
export class ApiService {
  constructor(private httpClient: HttpClient) {}

  login(email: string, password: string): Observable<string> {
    return MOCKED
      ? buildMocked(MOCK_LOGIN_RESPONSE)
      : this.httpClient.post<string>(API_URL + '/auth/login', {
          email,
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
      : this.httpClient.get<UserData>(API_URL + '/user');
  }
}
