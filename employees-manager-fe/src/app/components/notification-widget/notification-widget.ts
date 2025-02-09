import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { AppNotification } from '../../types/model';
import { ApiService } from '../../service/api.service';
import { NotificationType } from '../../types/enums';
import { Router, RouterLink } from '@angular/router';
import { MatBadgeModule } from '@angular/material/badge';

@Component({
  selector: 'notification-widget',
  standalone: true,
  imports: [
    CommonModule,
    MatMenuModule,
    MatIconModule,
    MatButtonModule,
    RouterLink,
    MatBadgeModule,
  ],
  templateUrl: './notification-widget.html',
  styleUrl: './notification-widget.scss',
  encapsulation: ViewEncapsulation.None,
})
export class NotificationWidgetComponent implements OnInit {
  maxNotifications = 5;
  notifications: AppNotification[] = [];

  constructor(private apiService: ApiService, private router: Router) {}

  ngOnInit(): void {
    this.loadNotifications();
  }

  loadNotifications() {
    this.apiService.getUnreadNotifications().subscribe({
      next: (notifications: AppNotification[]) => {
        this.notifications = notifications;
        this.maxNotifications = Math.min(this.notifications.length, 5);
      },
    });
  }

  notificationTypeToTitle(notificationType: NotificationType): string {
    if (notificationType == NotificationType.InviteAddCompany) {
      return 'Company invitation';
    } else if (notificationType == NotificationType.InviteAddCompanyAnswer) {
      return 'Company invitation answer';
    } else {
      return '';
    }
  }

  notificationTypeToIcon(notificationType: NotificationType): string {
    if (notificationType == NotificationType.InviteAddCompany) {
      return 'work';
    } else if (notificationType == NotificationType.InviteAddCompanyAnswer) {
      return 'info';
    } else {
      return '';
    }
  }
}
