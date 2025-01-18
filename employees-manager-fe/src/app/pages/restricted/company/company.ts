import { Component, OnInit, ViewChild } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute } from '@angular/router';
import { forkJoin } from 'rxjs';
import { UserService } from '../../../service/user.service';
import {
  CompanyInfo,
  InvitedUserInCompanyInfo,
  InviteUserInCompany,
  UserData,
  UserInCompanyInfo,
} from '../../../types/model';
import {
  FormBuilder,
  FormGroup,
  FormsModule,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatFormFieldModule } from '@angular/material/form-field';
import { ApiService } from '../../../service/api.service';
import { CompanyRole } from '../../../types/enums';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatIconModule } from '@angular/material/icon';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import {
  MatButtonToggleChange,
  MatButtonToggleModule,
} from '@angular/material/button-toggle';
import { MatMenuModule } from '@angular/material/menu';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { ToastrService } from 'ngx-toastr';
import { MatDialog } from '@angular/material/dialog';
import { InviteUserInCompanyDialogComponent } from './invite-user/invite-user-modal';
@Component({
  selector: 'company-page',
  templateUrl: './company.html',
  styleUrls: ['./company.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatFormFieldModule,
    MatSelectModule,
    MatInputModule,
    MatTableModule,
    MatProgressBarModule,
    MatIconModule,
    MatSortModule,
    MatPaginatorModule,
    MatButtonToggleModule,
    ReactiveFormsModule,
    MatMenuModule,
    MatInputModule,
  ],
})
export class CompanyPageComponent implements OnInit {
  CompanyRole = CompanyRole;

  loading: boolean = false;
  userData: UserData | null = null;
  companyId: string | null = null;
  companies: CompanyInfo[] = [];
  usersInCompany: UserInCompanyInfo[] = [];
  usersTableDataSource: MatTableDataSource<UserInCompanyInfo> =
    new MatTableDataSource<UserInCompanyInfo>([]);
  readonly userFilterForm: FormGroup;

  pendingUsers: InvitedUserInCompanyInfo[] = [];
  pendingUsersTableDataSource: MatTableDataSource<InvitedUserInCompanyInfo> =
    new MatTableDataSource<InvitedUserInCompanyInfo>([]);
  readonly pendingUserFilterForm: FormGroup;

  changeJobTitleForm: FormGroup = this.formBuilder.group({
    jobTitle: ['', Validators.required],
  });

  displayedUsersInfoColumns: string[] = [
    'id',
    'username',
    'name',
    'surname',
    'jobTitle',
    'role',
    'manager',
    'actionMenu',
  ];

  displayedPendingUsersInfoColumns: string[] = [
    'id',
    'username',
    'jobTitle',
    'role',
    'actionMenu',
  ];

  @ViewChild(MatSort, { static: false }) sort!: MatSort;
  @ViewChild(MatPaginator, { static: false }) paginator!: MatPaginator;

  constructor(
    private route: ActivatedRoute,
    private userService: UserService,
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    private toastr: ToastrService,
    private dialog: MatDialog
  ) {
    this.userFilterForm = formBuilder.group({
      valueString: '',
      activeUser: null,
      role: null,
      manager: null,
    });
    this.userFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLocaleLowerCase(),
        activeUser:
          value.activeUser === null || value.activeUser.length === 0
            ? null
            : value.activeUser[value.activeUser.length - 1] === 'true',
        role:
          value.role === null || value.role.length === 0
            ? null
            : value.role[value.role.length - 1],
        manager:
          value.manager === null || value.manager.length === 0
            ? null
            : value.manager[value.manager.length - 1] === 'true',
      } as string;
      this.usersTableDataSource.filter = filter;
    });

    this.pendingUserFilterForm = formBuilder.group({
      valueString: '',
      role: null,
    });
    this.pendingUserFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLocaleLowerCase(),
        role:
          value.role === null || value.role.length === 0
            ? null
            : value.role[value.role.length - 1],
      } as string;
      this.pendingUsersTableDataSource.filter = filter;
    });
  }

  ngOnInit(): void {
    this.route.queryParamMap.subscribe((params) => {
      this.companyId = params.get('companyId');
      this.loadData();
    });
  }

  loadData() {
    this.loading = true;

    forkJoin({
      userData: this.userService.fetchUserData(),
      companies: this.apiService.getUserCompanies(),
    }).subscribe({
      next: (response) => {
        this.userData = response.userData;
        this.companies = response.companies.filter((company) => {
          return (
            company.role === CompanyRole.Admin ||
            company.role === CompanyRole.Owner
          );
        });
        this.loading = false;
      },
      error: () => {
        this.userData = null;
        this.loading = false;
      },
    });

    if (this.companyId !== null) {
      forkJoin({
        users: this.apiService.getUsersInCompany(this.companyId),
        pendingUsers: this.apiService.getPendingUsersInCompany(this.companyId),
      }).subscribe({
        next: (response) => {
          this.usersInCompany = response.users;
          this.usersTableDataSource = new MatTableDataSource(
            this.usersInCompany
          );
          setTimeout(() => {
            this.usersTableDataSource.filterPredicate = (data, filter: any) => {
              const roleFilter =
                filter.role === null ? true : data.role === filter.role;
              const managerFilter =
                filter.manager === null
                  ? true
                  : data.managementTeam === filter.manager;
              const idFilter = data.userId
                .toLocaleLowerCase()
                .includes(filter.valueString);
              const usernameFilter = data.userUsername
                .toLocaleLowerCase()
                .trim()
                .includes(filter.valueString);
              const nameFilter = data.userName
                .toLocaleLowerCase()
                .trim()
                .includes(filter.valueString);
              const surnameFilter = data.userSurname
                .toLocaleLowerCase()
                .trim()
                .includes(filter.valueString);

              return (
                roleFilter &&
                managerFilter &&
                (idFilter || usernameFilter || nameFilter || surnameFilter)
              );
            };
            this.usersTableDataSource.sort = this.sort;
            this.usersTableDataSource.paginator = this.paginator;
          });

          this.pendingUsers = response.pendingUsers;
          this.pendingUsersTableDataSource = new MatTableDataSource(
            this.pendingUsers
          );
          setTimeout(() => {
            this.pendingUsersTableDataSource.filterPredicate = (
              data,
              filter: any
            ) => {
              const roleFilter =
                filter.role === null ? true : data.role === filter.role;
              const idFilter = data.userId
                .toLocaleLowerCase()
                .includes(filter.valueString);
              const usernameFilter = data.username
                .toLocaleLowerCase()
                .trim()
                .includes(filter.valueString);

              return roleFilter && (idFilter || usernameFilter);
            };
            this.pendingUsersTableDataSource.sort = this.sort;
            this.pendingUsersTableDataSource.paginator = this.paginator;
          });
        },
        error: () => {
          this.usersInCompany = [];
          this.usersTableDataSource = new MatTableDataSource(
            this.usersInCompany
          );
          setTimeout(() => {
            this.usersTableDataSource.sort = this.sort;
            this.usersTableDataSource.paginator = this.paginator;
          });

          this.pendingUsers = [];
          this.pendingUsersTableDataSource = new MatTableDataSource(
            this.pendingUsers
          );
          setTimeout(() => {
            this.pendingUsersTableDataSource.sort = this.sort;
            this.pendingUsersTableDataSource.paginator = this.paginator;
          });
        },
      });
    }
  }

  openInviteUserInCompanyDialog() {
    this.dialog
      .open(InviteUserInCompanyDialogComponent, {
        width: '40rem',
        data: {
          companyId: this.companyId,
          role: this.companies.filter(
            (company) => company.id == this.companyId
          )[0].role,
        },
      })
      .afterClosed()
      .subscribe({
        next: (data: InviteUserInCompany) => {
          if (data !== undefined) {
            this.apiService
              .inviteUserInCompany(
                this.companyId!,
                data.userId,
                data.role,
                data.jobTitle
              )
              .subscribe({
                next: () => {
                  this.loadData();
                  this.toastr.success('User invited to company', 'Sent', {
                    timeOut: 5000,
                    progressBar: true,
                  });
                },
              });
          }
        },
      });
  }

  onManagerFilterChange(event: MatButtonToggleChange) {
    const toggle = event.source;
    if (toggle && event.value.some((item: string) => item === toggle.value)) {
      toggle.buttonToggleGroup.value = [toggle.value];
    }
  }

  onChangeRole(element: UserInCompanyInfo, newRole: CompanyRole) {
    this.apiService
      .changeUserCompanyRole(element.companyId, element.userId, newRole)
      .subscribe({
        next: () => {
          this.toastr.success(
            'User with id ' + element.userId + ' changed to ' + newRole,
            'Role changed',
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

  setAsManager(element: UserInCompanyInfo) {
    this.apiService
      .setUserCompanyManager(element.companyId, element.userId)
      .subscribe({
        next: () => {
          this.toastr.success(
            'User with id ' + element.userId + ' set as company manager',
            'Manager changed',
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

  unsetAsManager(element: UserInCompanyInfo) {
    this.apiService
      .unsetUserCompanyManager(element.companyId, element.userId)
      .subscribe({
        next: () => {
          this.toastr.success(
            'User with id ' + element.userId + ' unset as company manager',
            'Manager changed',
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

  changeJobTitle(element: UserInCompanyInfo) {
    this.apiService
      .changeUserJobTitle(
        element.companyId,
        element.userId,
        this.changeJobTitleForm.value['jobTitle']
      )
      .subscribe({
        next: () => {
          this.toastr.success(
            'Changed job title for user ' + element.userId,
            'Job title changed',
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

  removeFromCompany(element: UserInCompanyInfo) {
    this.apiService
      .removeUserFromCompany(element.companyId, element.userId)
      .subscribe({
        next: () => {
          this.toastr.success(
            'User with id ' + element.userId + ' removed from company',
            'User removed',
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

  isCompanyAdminOrHigher(): boolean {
    if (this.companyId === null) {
      return false;
    } else {
      const userRole = this.companies.filter(
        (elem) => elem.id === this.companyId
      )[0].role;
      return userRole === CompanyRole.Admin || userRole === CompanyRole.Owner;
    }
  }

  isCompanyOwner(): boolean {
    if (this.companyId === null) {
      return false;
    } else {
      const userRole = this.companies.filter(
        (elem) => elem.id === this.companyId
      )[0].role;
      return userRole === CompanyRole.Owner;
    }
  }

  cancelInvitation(invitation: InvitedUserInCompanyInfo) {
    this.apiService
      .cancelInvitation(invitation.companyId, invitation.notificationId)
      .subscribe({
        next: () => {
          this.toastr.success(
            `Canceled invitation for user ${invitation.username}`,
            'Invitation canceled',
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
}
