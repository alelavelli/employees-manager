import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewChild, ViewEncapsulation } from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatMenuModule } from '@angular/material/menu';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { ApiService } from '../../../service/api.service';
import { ActivatedRoute } from '@angular/router';
import { AppNotification } from '../../../types/model';
import { ToastrService } from 'ngx-toastr';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
import { NotificationType } from '../../../types/enums';

@Component({
  selector: 'notifications-page',
  templateUrl: './notifications.html',
  styleUrls: ['./notifications.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatProgressBarModule,
    MatTableModule,
    MatIconModule,
    MatSortModule,
    MatPaginatorModule,
    MatFormFieldModule,
    MatInputModule,
    ReactiveFormsModule,
    MatMenuModule,
    MatButtonToggleModule,
  ],
  encapsulation: ViewEncapsulation.None,
})
export class NotificationsPageComponent implements OnInit {
  NotificationType = NotificationType;
  loading: boolean = false;
  notificationId: string | null = null;

  notifications: AppNotification[] = [];
  notificationsTableDataSource: MatTableDataSource<AppNotification> =
    new MatTableDataSource<AppNotification>([]);
  readonly notificationsFilterForm: FormGroup;

  displayedNotificationsInfoColumns: string[] = [
    'notificationType',
    'message',
    'actionMenu',
  ];

  @ViewChild(MatSort, { static: false }) sort!: MatSort;
  @ViewChild(MatPaginator, { static: false }) paginator!: MatPaginator;

  constructor(
    private route: ActivatedRoute,
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    private toastr: ToastrService
  ) {
    this.notificationsFilterForm = formBuilder.group({
      valueString: '',
      notificationType: null,
      notificationId: null,
    });
    this.notificationsFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString:
          value.valueString === null
            ? ''
            : value.valueString.trim().toLocaleLowerCase(),
        notificationId: value.notificationId,
        notificationType:
          value.notificationType === null || value.notificationType.length === 0
            ? null
            : value.notificationType[value.notificationType.length - 1],
      } as string;
      this.notificationsTableDataSource.filter = filter;
    });
  }

  ngOnInit(): void {
    this.route.queryParamMap.subscribe((params) => {
      this.notificationId = params.get('notificationId');
      this.loadNotifications();
    });
  }

  loadNotifications() {
    this.loading = true;

    this.apiService.getUnreadNotifications().subscribe({
      next: (notifications: AppNotification[]) => {
        this.notifications = notifications;

        this.notificationsTableDataSource = new MatTableDataSource(
          this.notifications
        );
        setTimeout(() => {
          this.notificationsTableDataSource.filterPredicate = (
            data,
            filter: any
          ) => {
            const typeFilter =
              filter.notificationType === null
                ? true
                : data.notificationType === filter.notificationType;
            const idFilter =
              this.notificationId === null
                ? true
                : data.id.toLocaleLowerCase().includes(this.notificationId);

            const messageFilter = data.message
              .toLocaleLowerCase()
              .trim()
              .includes(filter.valueString);

            return typeFilter && idFilter && messageFilter;
          };
          this.notificationsFilterForm.setValue({
            notificationId: this.notificationId,
            valueString: '',
            notificationType: null,
          });
          this.notificationsTableDataSource.sort = this.sort;
          this.notificationsTableDataSource.paginator = this.paginator;
        });
        this.loading = false;
      },
      error: () => {
        this.notifications = [];
        this.notificationsTableDataSource = new MatTableDataSource(
          this.notifications
        );
        setTimeout(() => {
          this.notificationsTableDataSource.sort = this.sort;
          this.notificationsTableDataSource.paginator = this.paginator;
        });
        this.loading = false;
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

  acceptInviteAddCompany(notification: AppNotification) {
    this.apiService.acceptInviteAddCompany(notification.id).subscribe({
      next: () => {
        this.toastr.success('Accepted invite to company', 'Success', {
          timeOut: 5000,
          progressBar: true,
        });
        this.loadNotifications();
      },
    });
  }
  declineInviteAddCompany(notification: AppNotification) {
    this.apiService.declineInviteAddCompany(notification.id).subscribe({
      next: () => {
        this.toastr.success('Declined invite to company', 'Success', {
          timeOut: 5000,
          progressBar: true,
        });
        this.loadNotifications();
      },
    });
  }

  setAsRead(notification: AppNotification) {
    this.apiService.setNotificationAsRead(notification.id).subscribe({
      next: () => {
        this.toastr.success('Notification marked as read', 'Success', {
          timeOut: 5000,
          progressBar: true,
        });
        this.loadNotifications();
      },
    });
  }
}
