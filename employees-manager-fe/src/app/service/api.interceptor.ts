import {
  HttpHandlerFn,
  HttpHeaders,
  HttpInterceptorFn,
  HttpRequest,
} from '@angular/common/http';
import { inject } from '@angular/core';
import { UserService } from './user.service';

const excludedApiList = ['/login'];

export const authInterceptor: HttpInterceptorFn = (
  request: HttpRequest<any>,
  next: HttpHandlerFn
) => {
  const userService = inject(UserService);

  let req = request;

  if (
    request.url.includes('/api/') &&
    excludedApiList.every((e) => !request.url.includes(e))
  ) {
    req = request.clone({
      headers: new HttpHeaders({
        Authorization: `Bearer ${userService.getJwtToken()}`,
      }),
    });
  }

  return next(req);
};
