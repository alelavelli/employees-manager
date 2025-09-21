import { inject } from '@angular/core';
import {
  ActivatedRouteSnapshot,
  CanActivateFn,
  Router,
  RouterStateSnapshot,
} from '@angular/router';
import { UserService } from '../service/user.service';
import { map } from 'rxjs';

export const AuthAdminGuard: CanActivateFn = (
  next: ActivatedRouteSnapshot,
  state: RouterStateSnapshot
) => {
  const router = inject(Router);
  const userService = inject(UserService);

  return userService.isPlatformAdmin().pipe(
    map((isAdmin) => {
      if (isAdmin && userService.isAuthenticated()) {
        return true;
      } else {
        return router.parseUrl('/does-not-exist');
      }
    })
  );
};
