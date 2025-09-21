import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewChild, ViewEncapsulation } from '@angular/core';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { ApiService } from '../../../service/api.service';
import {
  AdminPanelOverview,
  AdminPanelUserInfo,
  CreateUserParameters,
} from '../../../types/model';
import { forkJoin } from 'rxjs';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { MatIconModule } from '@angular/material/icon';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatDialog } from '@angular/material/dialog';
import {
  MatButtonToggleChange,
  MatButtonToggleModule,
} from '@angular/material/button-toggle';
import { FormBuilder, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatMenuModule } from '@angular/material/menu';
import { ToastrService } from 'ngx-toastr';
import { NewUserDialogComponent } from './new-user-modal/new-user-modal';
import { ConfirmDialogComponent } from '../../../components/confirm-modal/confirm-modal';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'admin-page',
  templateUrl: './admin.html',
  styleUrls: ['./admin.scss'],
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
    MatButtonToggleModule,
    ReactiveFormsModule,
    MatMenuModule,
    MatButtonModule,
  ],
  encapsulation: ViewEncapsulation.None,
})
export class AdminPageComponent implements OnInit {
  loading: boolean = false;

  overview: AdminPanelOverview | null = null;

  users: AdminPanelUserInfo[] = [];
  usersTableDataSource: MatTableDataSource<AdminPanelUserInfo> =
    new MatTableDataSource<AdminPanelUserInfo>([]);
  readonly userFilterForm: FormGroup;

  displayedUsersInfoColumns: string[] = [
    'id',
    'username',
    'email',
    'name',
    'surname',
    'platformAdmin',
    'active',
    'totalCompanies',
    'actionMenu',
  ];

  @ViewChild(MatSort, { static: false }) sort!: MatSort;
  @ViewChild(MatPaginator, { static: false }) paginator!: MatPaginator;

  constructor(
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    private toastr: ToastrService,
    private dialog: MatDialog
  ) {
    this.userFilterForm = formBuilder.group({
      valueString: '',
      activeUser: null,
      platformAdmin: null,
    });
    this.userFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLowerCase(),
        activeUser:
          value.activeUser === null || value.activeUser.length === 0
            ? null
            : value.activeUser[value.activeUser.length - 1] === 'true',
        platformAdmin:
          value.platformAdmin === null || value.platformAdmin.length === 0
            ? null
            : value.platformAdmin[value.platformAdmin.length - 1] === 'true',
      } as string;
      this.usersTableDataSource.filter = filter;
    });
  }

  ngOnInit(): void {
    this.loadData();
  }

  loadData() {
    this.loading = true;

    forkJoin({
      overview: this.apiService.getAdminPanelOverview(),
      users: this.apiService.getAdminUsersInfo(),
    }).subscribe({
      next: (response) => {
        this.overview = response.overview;
        this.users = response.users;
        this.usersTableDataSource = new MatTableDataSource(this.users);
        setTimeout(() => {
          this.usersTableDataSource.filterPredicate = (data, filter: any) => {
            const activeUserFilter =
              filter.activeUser === null
                ? true
                : data.active === filter.activeUser;

            const platformAdminFilter =
              filter.platformAdmin === null
                ? true
                : data.platformAdmin === filter.platformAdmin;

            const idFilter = data.id
              .toLocaleLowerCase()
              .includes(filter.valueString);
            const usernameFilter = data.username
              .toLocaleLowerCase()
              .trim()
              .includes(filter.valueString);
            const emailFilter = data.email
              .toLocaleLowerCase()
              .trim()
              .includes(filter.valueString);
            const nameFilter = data.name
              .toLocaleLowerCase()
              .trim()
              .includes(filter.valueString);
            const surnameFilter = data.surname
              .toLocaleLowerCase()
              .trim()
              .includes(filter.valueString);

            return (
              activeUserFilter &&
              platformAdminFilter &&
              (idFilter ||
                usernameFilter ||
                emailFilter ||
                nameFilter ||
                surnameFilter)
            );
          };
          this.usersTableDataSource.sort = this.sort;
          this.usersTableDataSource.paginator = this.paginator;
        });
        this.loading = false;
      },
      error: () => {
        this.overview = null;
        this.users = [];
        this.usersTableDataSource = new MatTableDataSource(this.users);
        setTimeout(() => {
          this.usersTableDataSource.sort = this.sort;
          this.usersTableDataSource.paginator = this.paginator;
        });
        this.loading = false;
      },
    });
  }

  onActiveUserFilterChange(event: MatButtonToggleChange) {
    const toggle = event.source;
    if (toggle && event.value.some((item: string) => item === toggle.value)) {
      toggle.buttonToggleGroup.value = [toggle.value];
    }
  }

  onPlatformAdminFilterChange(event: MatButtonToggleChange) {
    const toggle = event.source;
    if (toggle && event.value.some((item: string) => item === toggle.value)) {
      toggle.buttonToggleGroup.value = [toggle.value];
    }
  }

  setPlatformAdminUser(element: AdminPanelUserInfo) {
    this.apiService.setPlatformAdminUser(element.id).subscribe({
      next: () => {
        this.toastr.success(
          'User with id ' + element.id + ' set as platform admin',
          'Set platform admin',
          {
            timeOut: 5000,
            progressBar: true,
          }
        );
        this.loadData();
      },
      error: () => {},
    });
  }

  unsetPlatformAdminUser(element: AdminPanelUserInfo) {
    this.apiService.unsetPlatformAdminUser(element.id).subscribe({
      next: () => {
        this.toastr.success(
          'User with id ' + element.id + ' unset as platform admin',
          'Unset platform admin',
          {
            timeOut: 5000,
            progressBar: true,
          }
        );
        this.loadData();
      },
      error: () => {},
    });
  }

  activateUser(element: AdminPanelUserInfo) {
    this.apiService.activateUser(element.id).subscribe({
      next: () => {
        this.toastr.success(
          'User with id ' + element.id + ' activated',
          'Activated',
          {
            timeOut: 5000,
            progressBar: true,
          }
        );
        this.loadData();
      },
      error: () => {},
    });
  }

  deactivateUser(element: any) {
    this.apiService.deactivateUser(element.id).subscribe({
      next: () => {
        this.toastr.success(
          'User with id ' + element.id + ' deactivated',
          'Deactivated',
          {
            timeOut: 5000,
            progressBar: true,
          }
        );
        this.loadData();
      },
      error: () => {},
    });
  }

  deleteUser(element: any) {
    this.dialog
      .open(ConfirmDialogComponent, {
        data: {
          title: 'Delete user',
          content:
            'Are you sure you want to delete user with id <b>' +
            element.id +
            '</b>?',
        },
      })
      .afterClosed()
      .subscribe({
        next: (result) => {
          if (result) {
            this.apiService.deleteUser(element.id).subscribe({
              next: () => {
                this.toastr.success(
                  'User with id ' + element.id + ' deleted',
                  'Deleted',
                  {
                    timeOut: 5000,
                    progressBar: true,
                  }
                );
                this.loadData();
              },
              error: () => {},
            });
          }
        },
      });
  }

  openCreateUserDialog() {
    this.dialog
      .open(NewUserDialogComponent, {
        width: '40rem',
        data: {},
      })
      .afterClosed()
      .subscribe({
        next: (newUser: CreateUserParameters | undefined) => {
          if (newUser !== undefined) {
            this.apiService.createUser(newUser).subscribe({
              next: (userId: string) => {
                this.loadData();
                this.toastr.success(
                  'New user created with id ' + userId,
                  'Sent',
                  {
                    timeOut: 5000,
                    progressBar: true,
                  }
                );
              },
            });
          }
        },
      });
  }
}
