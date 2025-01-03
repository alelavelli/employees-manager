import {
  HttpHandlerFn,
  HttpHeaders,
  HttpInterceptorFn,
  HttpRequest,
} from '@angular/common/http';
import { inject } from '@angular/core';
import { UserService } from './user.service';
import { catchError } from 'rxjs';
import { Router } from '@angular/router';
import { ToastrService } from 'ngx-toastr';

const excludedApiList = ['/login'];

export const authInterceptor: HttpInterceptorFn = (
  request: HttpRequest<any>,
  next: HttpHandlerFn
) => {
  const userService = inject(UserService);
  const router = inject(Router);
  const toastr = inject(ToastrService);

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
  return next(req).pipe(
    catchError((err) => {
      if (err.status === 401) {
        toastr.error('Invalid credentials', 'Authorization Error', {
          timeOut: 5000,
          progressBar: true,
        });
      }

      if (err.error && err.error.errorCode && err.error.errorMessage) {
        toastr.error(err.error.errorMessage, `ERROR - ${err.error.errorCode}`, {
          timeOut: 5000,
          progressBar: true,
        });
      }

      // TODO: handle when the server is unreachable but a jwt is present in local storage

      if (err.status === 0) {
        toastr.error(
          'Could not reach the remote server',
          `SERVER UNREACHABLE`,
          {
            timeOut: 5000,
            progressBar: true,
          }
        );
      }

      throw err;
    })
  );
};
