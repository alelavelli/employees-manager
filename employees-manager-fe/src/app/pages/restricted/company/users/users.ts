import { CommonModule } from '@angular/common';
import {
  Component,
  Input,
  OnInit,
  QueryList,
  ViewChildren,
} from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { CompanyRole } from '../../../../types/enums';
import {
  CompanyInfo,
  InvitedUserInCompanyInfo,
  InviteUserInCompany,
  UserData,
  UserInCompanyInfo,
} from '../../../../types/model';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import {
  FormBuilder,
  FormGroup,
  FormsModule,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { MatInputModule } from '@angular/material/input';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatIconModule } from '@angular/material/icon';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import {
  MatButtonToggleChange,
  MatButtonToggleModule,
} from '@angular/material/button-toggle';
import { MatMenuModule } from '@angular/material/menu';
import { MatTabsModule } from '@angular/material/tabs';
import { MatAutocompleteModule } from '@angular/material/autocomplete';
import { MatListModule } from '@angular/material/list';
import { ApiService } from '../../../../service/api.service';
import { ToastrService } from 'ngx-toastr';
import { MatDialog } from '@angular/material/dialog';
import { InviteUserInCompanyDialogComponent } from '../invite-user/invite-user-modal';
import { forkJoin } from 'rxjs';

@Component({
  selector: 'company-users',
  templateUrl: './users.html',
  styleUrls: ['./users.scss'],
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
    MatTabsModule,
    MatAutocompleteModule,
    MatListModule,
    FormsModule,
    MatButtonModule,
  ],
})
export class CompanyUsers implements OnInit {
  @Input() userData!: UserData;
  @Input() company!: CompanyInfo;
  @Input() usersInCompany!: UserInCompanyInfo[];

  CompanyRole = CompanyRole;

  loading: boolean = false;

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
    this.loadData();
  }

  loadData() {
    forkJoin({
      pendingUsers: this.apiService.getPendingUsersInCompany(this.company.id),
    }).subscribe({
      next: (response) => {
        this.usersTableDataSource = new MatTableDataSource(this.usersInCompany);
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
          this.usersTableDataSource.sort = this.sort.toArray()[0];
          this.usersTableDataSource.paginator = this.paginator.toArray()[0];
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
          this.pendingUsersTableDataSource.sort = this.sort.toArray()[1];
          this.pendingUsersTableDataSource.paginator =
            this.paginator.toArray()[1];
        });
      },
      error: () => {
        this.usersTableDataSource = new MatTableDataSource(this.usersInCompany);
        setTimeout(() => {
          this.usersTableDataSource.sort = this.sort.toArray()[0];
          this.usersTableDataSource.paginator = this.paginator.toArray()[0];
        });

        this.pendingUsers = [];
        this.pendingUsersTableDataSource = new MatTableDataSource(
          this.pendingUsers
        );
        setTimeout(() => {
          this.pendingUsersTableDataSource.sort = this.sort.toArray()[1];
          this.pendingUsersTableDataSource.paginator =
            this.paginator.toArray()[1];
        });
      },
    });
  }

  openInviteUserInCompanyDialog() {
    this.dialog
      .open(InviteUserInCompanyDialogComponent, {
        width: '40rem',
        data: {
          companyId: this.company.id,
          role: this.company.role,
        },
      })
      .afterClosed()
      .subscribe({
        next: (data: InviteUserInCompany) => {
          if (data !== undefined) {
            this.apiService
              .inviteUserInCompany(
                this.company.id,
                data.userId,
                data.role,
                data.jobTitle,
                data.projectIds
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
    return (
      this.company.role === CompanyRole.Admin ||
      this.company.role === CompanyRole.Owner
    );
  }

  isCompanyOwner(): boolean {
    return this.company.role === CompanyRole.Owner;
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
