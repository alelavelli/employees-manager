import { Routes } from '@angular/router';
import { environment } from '../environments/environment';
import { MaintenancePageComponent } from './pages/maintenance/maintenance.component';
import { LayoutType } from './types/enums';
import { SplashPageComponent } from './pages/splash/splash.component';
import { HomePageComponent } from './pages/restricted/home/home';
import { LoginPageComponent } from './pages/guest/login/login.component';
import { Page1PageComponent } from './pages/restricted/page1/page1';
import { Page2PageComponent } from './pages/restricted/page2/page2';
import { TypographyPageComponent } from './pages/restricted/typography/typography';
import { ServicePageComponent } from './pages/restricted/service/service';
import { AuthGuard } from './guards/auth.guard';
import { PipePageComponent } from './pages/restricted/pipe/pipe';
import { UnauthorizedPageComponent } from './pages/unauthorized/unauthorized.component';

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
        data: { layoutType: LayoutType.Guest },
      },
      {
        path: 'home',
        canActivate: [AuthGuard],
        component: HomePageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: 'page1',
        canActivate: [AuthGuard],
        component: Page1PageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: 'page2',
        canActivate: [AuthGuard],
        component: Page2PageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: 'typography',
        canActivate: [AuthGuard],
        component: TypographyPageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: 'service',
        canActivate: [AuthGuard],
        component: ServicePageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: 'pipe',
        canActivate: [AuthGuard],
        component: PipePageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
      {
        path: '**',
        component: UnauthorizedPageComponent,
        data: { layoutType: LayoutType.Restricted },
      },
    ];
