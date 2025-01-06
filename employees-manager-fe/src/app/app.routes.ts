import { Routes } from '@angular/router';
import { environment } from '../environments/environment';
import { MaintenancePageComponent } from './pages/maintenance/maintenance.component';
import { LayoutType } from './types/enums';
import { SplashPageComponent } from './pages/splash/splash.component';
import { HomePageComponent } from './pages/restricted/home/home';
import { LoginPageComponent } from './pages/guest/login/login.component';
import { AuthGuard } from './guards/auth.guard';
import { AdminPageComponent } from './pages/restricted/admin/admin';
import { NotificationsPageComponent } from './pages/restricted/notifications/notifications';
import { AuthAdminGuard } from './guards/auth_admin.guard';
import { DoesNotExistPageComponent } from './pages/does-not-exist/does-not-exist.component';
import { GuestGuard } from './guards/auth_guest.guard';

export const routes: Routes = environment.maintenance
  ? [
      {
        path: '**',
        component: MaintenancePageComponent,
        data: { layoutType: LayoutType.Maintenance },
      },
    ]
  : [
      {
        path: '',
        component: SplashPageComponent,
        data: { layoutType: LayoutType.Splash },
      },
      {
        path: 'login',
        component: LoginPageComponent,
        canActivate: [GuestGuard],
        data: { layoutType: LayoutType.Guest },
      },
      {
        path: 'home',
        canActivate: [AuthGuard],
        component: HomePageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: 'admin',
        canActivate: [AuthGuard, AuthAdminGuard],
        component: AdminPageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: 'notifications',
        canActivate: [AuthGuard],
        component: NotificationsPageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: '**',
        component: DoesNotExistPageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
    ];
