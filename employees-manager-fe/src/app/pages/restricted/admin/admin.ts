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
import {
  AbstractControlOptions,
  FormBuilder,
  FormControl,
  FormGroup,
  FormsModule,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
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
    FormsModule,
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

  setUserPasswordForm: FormGroup = this.formBuilder.group(
    {
      password: new FormControl('', [
        Validators.required,
        Validators.minLength(8),
        Validators.pattern(/[a-z]/), // At least 1 lowercase
        Validators.pattern(/[A-Z]/), // At least 1 uppercase
        Validators.pattern(/\d/), // At least 1 digit
        Validators.pattern(/[^a-zA-Z0-9]/), // At least 1 symbol
      ]),
      confirmPassword: new FormControl('', [Validators.required]),
    },
    <AbstractControlOptions>{ validators: [this.passwordMatchValidator] }
  );

  passwordConstraints = [
    { respected: false, message: 'at least 8 characters long' },
    { respected: false, message: 'at least one lowercase letter' },
    { respected: false, message: 'at least one uppercase letter' },
    { respected: false, message: 'at least one digit' },
    { respected: false, message: 'at least one symbol' },
    { respected: false, message: 'passwords must match' },
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

    this.setUserPasswordForm.valueChanges.subscribe((form) => {
      const password: string = form.password;
      this.passwordConstraints[0].respected = password.length >= 8;
      this.passwordConstraints[1].respected = password.match(/[a-z]/) !== null;
      this.passwordConstraints[2].respected = password.match(/[A-Z]/) !== null;
      this.passwordConstraints[3].respected = password.match(/\d/) !== null;
      this.passwordConstraints[4].respected =
        password.match(/[^a-zA-Z0-9]/) !== null;
      this.passwordConstraints[5].respected = password === form.confirmPassword;
    });
  }

  ngOnInit(): void {
    this.loadData();
  }

  passwordMatchValidator(formGroup: FormGroup) {
    const password: string = formGroup.get('password')!.value;
    const confirmPassword: string = formGroup.get('confirmPassword')!.value;

    if (password !== confirmPassword) {
      formGroup.get('confirmPassword')!.setErrors({ passwordMismatch: true });
    } else {
      formGroup.get('confirmPassword')!.setErrors(null);
    }
  }

  setUserPassword(user: AdminPanelUserInfo) {
    this.apiService
      .setUserPassword(user.id, this.setUserPasswordForm.value['password'])
      .subscribe({
        next: () => {
          this.toastr.success(
            'Updated password for user with id ' + user.id,
            'Update success',
            {
              timeOut: 5000,
              progressBar: true,
            }
          );
          this.clearSetUserPasswordForm();
        },
        error: () => {
          this.clearSetUserPasswordForm();
        },
      });
  }

  clearSetUserPasswordForm() {
    this.setUserPasswordForm.reset();
    for (let obj of this.passwordConstraints) {
      obj.respected = false;
    }
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
