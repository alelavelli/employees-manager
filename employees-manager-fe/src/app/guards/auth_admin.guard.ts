import { inject } from '@angular/core';
import {
  ActivatedRouteSnapshot,
  CanActivateFn,
  Router,
  RouterStateSnapshot,
} from '@angular/router';
import { UserService } from '../service/user.service';

export const AuthAdminGuard: CanActivateFn = (
  next: ActivatedRouteSnapshot,
  state: RouterStateSnapshot
) => {
  const router = inject(Router);
  const userService = inject(UserService);

  if (userService.isAuthenticated() && userService.isPlatformAdmin()) {
    return true;
  }

  return router.parseUrl('/does-not-exist');
};
