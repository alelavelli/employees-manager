import { Component, OnInit } from '@angular/core';
import { MatIconRegistry } from '@angular/material/icon';
import { DomSanitizer } from '@angular/platform-browser';
import { Event, ResolveEnd, Router, RouterOutlet } from '@angular/router';
import { LayoutType } from '../../types/enums';
import { APP_ICONS } from './app-icons';
import { CommonModule } from '@angular/common';
import { SplashLayoutComponent } from '../../layout/splash/splash-layout.component';
import { MaintenanceLayoutComponent } from '../../layout/maintenance/maintenance-layout.component';
import { RestrictedLayoutComponent } from '../../layout/restricted/restricted-layout';
import { GuestLayoutComponent } from '../../layout/guest/guest-layout.component';
import { EmptyLayoutComponent } from '../../layout/empty/empty-layout.component';
import { ThemeService } from '../../service/theme-service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  standalone: true,
  imports: [
    CommonModule,
    RouterOutlet,
    SplashLayoutComponent,
    MaintenanceLayoutComponent,
    RestrictedLayoutComponent,
    GuestLayoutComponent,
    EmptyLayoutComponent,
  ],
})
export class AppComponent implements OnInit {
  LayoutType = LayoutType;
  layoutType: LayoutType = LayoutType.Splash;

  constructor(
    private router: Router,
    private matIconRegistry: MatIconRegistry,
    private domSanitizer: DomSanitizer,
    private themeService: ThemeService
  ) {
    Object.entries(APP_ICONS).forEach(([k, v]) => {
      this.matIconRegistry.addSvgIcon(
        k,
        this.domSanitizer.bypassSecurityTrustResourceUrl(v)
      );
    });
  }

  ngOnInit(): void {
    this.initializePageChangeListener();
  }
  initializePageChangeListener() {
    this.router.events.subscribe({
      next: (event: Event) => {
        if (event instanceof ResolveEnd) {
          this.layoutType = event.state.root.firstChild?.data['layoutType'];
        }
      },
    });
  }
}
