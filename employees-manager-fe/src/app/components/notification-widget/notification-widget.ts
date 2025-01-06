import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatMenuModule } from '@angular/material/menu';
import { AppNotification } from '../../types/model';
import { ApiService } from '../../service/api.service';
import { NotificationType } from '../../types/enums';
import { Router, RouterLink } from '@angular/router';

@Component({
  selector: 'notification-widget',
  standalone: true,
  imports: [
    CommonModule,
    MatMenuModule,
    MatIconModule,
    MatButtonModule,
    RouterLink,
  ],
  templateUrl: './notification-widget.html',
  styleUrl: './notification-widget.scss',
  encapsulation: ViewEncapsulation.None,
})
export class NotificationWidgetComponent implements OnInit {
  notifications: AppNotification[] = [];

  constructor(private apiService: ApiService, private router: Router) {}

  ngOnInit(): void {
    this.loadNotifications();
  }

  loadNotifications() {
    this.apiService.getUnreadNotifications().subscribe({
      next: (notifications: AppNotification[]) => {
        this.notifications = notifications;
      },
    });
  }

  notificationTypeToTitle(notificationType: NotificationType): string {
    if (notificationType == NotificationType.InviteAddCompany) {
      return 'Company invitation';
    } else {
      return '';
    }
  }

  notificationTypeToIcon(notificationType: NotificationType): string {
    if (notificationType == NotificationType.InviteAddCompany) {
      return 'work';
    } else {
      return '';
    }
  }
}
