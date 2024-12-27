import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { environment } from '../../environments/environment';
import { MOCK_LOGIN_RESPONSE, buildMocked } from './mock';

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
}
