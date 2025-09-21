import { CommonModule } from '@angular/common';
import {
  Component,
  OnInit,
  QueryList,
  ViewChild,
  ViewChildren,
  ViewEncapsulation,
} from '@angular/core';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { ApiService } from '../../../service/api.service';
import {
  AdminCorporateGroupInfo,
  AdminPanelOverview,
  AdminPanelUserInfo,
  CreateCorporateGroupParameters,
  CreateUserParameters,
} from '../../../types/model';
import { forkJoin, map, Observable, of, startWith } from 'rxjs';
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
  AbstractControl,
  AbstractControlOptions,
  FormBuilder,
  FormControl,
  FormGroup,
  FormsModule,
  ReactiveFormsModule,
  ValidationErrors,
  ValidatorFn,
  Validators,
} from '@angular/forms';
import { MatMenuModule } from '@angular/material/menu';
import { ToastrService } from 'ngx-toastr';
import { NewUserDialogComponent } from './new-user-modal/new-user-modal';
import { ConfirmDialogComponent } from '../../../components/confirm-modal/confirm-modal';
import { MatButtonModule } from '@angular/material/button';
import { NewCorporateGroupDialogComponent } from './new-corporate-group-modal/new-corporate-group-modal';
import { MatAutocompleteModule } from '@angular/material/autocomplete';
import { MatOptionModule } from '@angular/material/core';

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
    MatAutocompleteModule,
    MatOptionModule,
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

  corporateGroups: AdminCorporateGroupInfo[] = [];
  corporateGroupsTableDataSource: MatTableDataSource<AdminCorporateGroupInfo> =
    new MatTableDataSource<AdminCorporateGroupInfo>([]);
  readonly corporateGroupsFilterForm: FormGroup;
  displayedCorporateGroupInfoColumns: string[] = [
    'id',
    'name',
    'active',
    'owner',
    'actionMenu',
  ];

  setCorporateGroupOwnerFormFilteredUsers: Observable<AdminPanelUserInfo[]>;
  setCorporateGroupOwnerFormCurrentUser: string | null = null;
  setCorporateGroupOwnerForm: FormGroup = this.formBuilder.group({
    username: ['', Validators.required, this.validUserValidator()],
  });

  @ViewChildren(MatSort) sort = new QueryList<MatSort>();
  @ViewChildren(MatPaginator) paginator = new QueryList<MatPaginator>();

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

    this.corporateGroupsFilterForm = formBuilder.group({
      valueString: '',
      activeCorporateGroup: null,
    });
    this.corporateGroupsFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLowerCase(),
        activeCorporateGroup:
          value.activeCorporateGroup === null ||
          value.activeCorporateGroup.length === 0
            ? null
            : value.activeCorporateGroup[
                value.activeCorporateGroup.length - 1
              ] === 'true',
      } as string;
      this.corporateGroupsTableDataSource.filter = filter;
    });

    this.setCorporateGroupOwnerFormFilteredUsers = of([]);
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

  validUserValidator(): ValidatorFn {
    return (control: AbstractControl): ValidationErrors | null => {
      const valid =
        this.users.filter((elem) => elem.username === control.value).length ==
        1;
      return of(valid ? null : { userDoesNotExist: { value: control.value } });
    };
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

  setCorporateGroupOwner(corporateGroup: AdminCorporateGroupInfo) {
    const user = this.users.filter(
      (elem) =>
        elem.username === this.setCorporateGroupOwnerForm.value['username']
    )[0]!;

    this.apiService
      .setCorporateGroupOwner(corporateGroup.id, user.id)
      .subscribe({
        next: () => {
          this.toastr.success(
            'User with id ' +
              user.id +
              ' is now corporate group owner of corporate group with id ' +
              corporateGroup.id,
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
    var sortIndex = 0;

    forkJoin({
      overview: this.apiService.getAdminPanelOverview(),
      users: this.apiService.getAdminUsersInfo(),
      corporateGroups: this.apiService.getAdminCorporateGroupsInfo(),
    }).subscribe({
      next: (response) => {
        this.overview = response.overview;
        this.users = response.users;
        this.usersTableDataSource = new MatTableDataSource(this.users);
        this.corporateGroups = response.corporateGroups;
        this.corporateGroupsTableDataSource = new MatTableDataSource(
          this.corporateGroups
        );
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
          this.usersTableDataSource.sort = this.sort.toArray()[sortIndex];
          this.usersTableDataSource.paginator =
            this.paginator.toArray()[sortIndex];
          sortIndex += 1;

          this.corporateGroupsTableDataSource.filterPredicate = (
            data,
            filter: any
          ) => {
            const activeUserFilter =
              filter.activeCorporateGroup === null
                ? true
                : data.active === filter.activeCorporateGroup;

            const idFilter = data.id
              .toLocaleLowerCase()
              .includes(filter.valueString);

            const nameFilter = data.name
              .toLocaleLowerCase()
              .trim()
              .includes(filter.valueString);

            return activeUserFilter && nameFilter;
          };
          this.corporateGroupsTableDataSource.sort =
            this.sort.toArray()[sortIndex];
          this.corporateGroupsTableDataSource.paginator =
            this.paginator.toArray()[sortIndex];
          sortIndex += 1;
        });

        this.setCorporateGroupOwnerFormFilteredUsers =
          this.setCorporateGroupOwnerForm.valueChanges.pipe(
            startWith(''),
            map((value: { username: string }) => {
              const username =
                typeof value.username === 'string'
                  ? value.username
                  : value.username!;
              return username
                ? this._filterCorporateGroupOwner(username as string)
                : this.users.slice();
            })
          );

        this.loading = false;
      },
      error: () => {
        this.overview = null;
        this.users = [];
        this.corporateGroups = [];
        this.usersTableDataSource = new MatTableDataSource(this.users);
        this.corporateGroupsTableDataSource = new MatTableDataSource(
          this.corporateGroups
        );
        setTimeout(() => {
          this.usersTableDataSource.sort = this.sort.toArray()[sortIndex];
          this.usersTableDataSource.paginator =
            this.paginator.toArray()[sortIndex];
          sortIndex += 1;
          this.corporateGroupsTableDataSource.sort =
            this.sort.toArray()[sortIndex];
          this.corporateGroupsTableDataSource.paginator =
            this.paginator.toArray()[sortIndex];
          sortIndex += 1;
        });
        this.loading = false;
      },
    });
  }

  private _filterCorporateGroupOwner(username: string): AdminPanelUserInfo[] {
    const filterValue = username.toLowerCase();

    return this.users.filter((option) =>
      option.username.toLowerCase().includes(filterValue)
    );
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

  openCreateCorporateGroupDialog() {
    this.dialog
      .open(NewCorporateGroupDialogComponent, {
        width: '40rem',
        data: {},
      })
      .afterClosed()
      .subscribe({
        next: (
          newCorporateGroup: CreateCorporateGroupParameters | undefined
        ) => {
          if (newCorporateGroup !== undefined) {
            this.apiService.createCorporateGroup(newCorporateGroup).subscribe({
              next: () => {
                this.loadData();
                this.toastr.success('New corporate group created', 'Sent', {
                  timeOut: 5000,
                  progressBar: true,
                });
              },
            });
          }
        },
      });
  }

  onActiveCorporateGroupFilterChange(event: MatButtonToggleChange) {
    const toggle = event.source;
    if (toggle && event.value.some((item: string) => item === toggle.value)) {
      toggle.buttonToggleGroup.value = [toggle.value];
    }
  }

  activateCorporateGroup(element: AdminCorporateGroupInfo) {
    this.apiService.activateCorporateGroup(element.id).subscribe({
      next: () => {
        this.toastr.success(
          'Corporate Group with id ' + element.id + ' activated',
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

  deactivateCorporateGroup(element: any) {
    this.apiService.deactivateCorporateGroup(element.id).subscribe({
      next: () => {
        this.toastr.success(
          'Corporate Group with id ' + element.id + ' deactivated',
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

  deleteCorporateGroup(element: any) {
    this.dialog
      .open(ConfirmDialogComponent, {
        data: {
          title: 'Delete corporate',
          content:
            'Are you sure you want to delete corporate group with id <b>' +
            element.id +
            '</b>?',
        },
      })
      .afterClosed()
      .subscribe({
        next: (result) => {
          if (result) {
            this.apiService.deleteCorporateGroup(element.id).subscribe({
              next: () => {
                this.toastr.success(
                  'Corporate Group with id ' + element.id + ' deleted',
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
}
