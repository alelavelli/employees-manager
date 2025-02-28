import { Component, OnInit, QueryList, ViewChildren } from '@angular/core';
import { AsyncPipe, CommonModule } from '@angular/common';
import { ActivatedRoute } from '@angular/router';
import { filter, forkJoin, map, Observable, of, startWith } from 'rxjs';
import { UserService } from '../../../service/user.service';
import {
  CompanyInfo,
  CompanyProjectInfo,
  InvitedUserInCompanyInfo,
  InviteUserInCompany,
  NewCompanyProject,
  NewProjectActivity,
  ProjectActivityInfo,
  UserData,
  UserInCompanyInfo,
} from '../../../types/model';
import {
  FormBuilder,
  FormControl,
  FormGroup,
  FormsModule,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatListModule } from '@angular/material/list';
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
import { MatTabsModule } from '@angular/material/tabs';
import { NewCompanyProjectDialogComponent } from './create-project/create-project-modal';
import { MatAutocompleteModule } from '@angular/material/autocomplete';
import { NewActivityDialogComponent } from './create-activity/create-activity-modal';

enum UserProjectAllocationViewMode {
  PROJECT = 'project',
  USER = 'user',
}

enum ActivityProjectAllocationViewMode {
  PROJECT = 'project',
  ACTIVITY = 'activity',
}

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
    MatTabsModule,
    MatAutocompleteModule,
    AsyncPipe,
    MatListModule,
    FormsModule,
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

  projects: CompanyProjectInfo[] = [];
  projectsTableDataSource: MatTableDataSource<CompanyProjectInfo> =
    new MatTableDataSource<CompanyProjectInfo>([]);
  readonly projectFilterForm: FormGroup;

  activities: ProjectActivityInfo[] = [];
  activitiesTableDataSource: MatTableDataSource<ProjectActivityInfo> =
    new MatTableDataSource<ProjectActivityInfo>([]);
  readonly activityFilterForm: FormGroup;

  changeJobTitleForm: FormGroup = this.formBuilder.group({
    jobTitle: ['', Validators.required],
  });

  editCompanyProjectForm: FormGroup = this.formBuilder.group({
    name: ['', Validators.required],
    code: ['', Validators.required],
    active: [null, Validators.required],
  });
  projectUnderEdit: string | null = null;

  editActivityForm: FormGroup = this.formBuilder.group({
    name: ['', Validators.required],
    description: [''],
  });
  activityUnderEdit: string | null = null;

  UserProjectAllocationViewMode = UserProjectAllocationViewMode;
  userProjectAllocationViewMode = UserProjectAllocationViewMode.PROJECT;
  projectUserAllocationViewForm: FormGroup = this.formBuilder.group({
    username: [''],
    project: [''],
  });
  userProjectAllocationModeUnderEdit: boolean = false;
  userProjectAllocationModeShow: boolean = false;
  userProjectAllocationViewFilteredProjects: Observable<CompanyProjectInfo[]>;
  userProjectAllocationViewFilteredUsers: Observable<UserInCompanyInfo[]>;
  currentAllocationProject: string | null = null;
  currentAllocationUser: string | null = null;
  usersAllocatedInProject: string[] = [];
  projectsAllocatedToUser: string[] = [];

  userProjectAllocationsForProjectForm: FormGroup = this.formBuilder.group({
    usernames: new FormControl([]),
  });
  userProjectAllocationsForUserForm: FormGroup = this.formBuilder.group({
    projects: new FormControl([]),
  });

  ActivityProjectAllocationViewMode = ActivityProjectAllocationViewMode;
  activityProjectAllocationViewMode = ActivityProjectAllocationViewMode.PROJECT;
  activityProjectAllocationForm: FormGroup = this.formBuilder.group({
    project: [''],
    activity: [''],
  });
  activityProjectAssignmentModeUnderEdit: boolean = false;
  activityProjectAssignmentModeShow: boolean = false;
  activityProjectAssignmentViewFilteredProjects: Observable<
    CompanyProjectInfo[]
  >;
  activityProjectAssignmentViewFilteredActivities: Observable<
    ProjectActivityInfo[]
  >;
  currentAssignmentProject: string | null = null;
  currentAssignmentActivity: string | null = null;
  activitiesAssignedInProject: string[] = [];
  projectsAssignedToActivity: string[] = [];

  assignmentsForProjectForm: FormGroup = this.formBuilder.group({
    activities: new FormControl([]),
  });
  assignmentsForActivityForm: FormGroup = this.formBuilder.group({
    projects: new FormControl([]),
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

  displayedProjectsInfoColumns: string[] = [
    'id',
    'name',
    'code',
    'active',
    'actionMenu',
  ];

  displayedActivityInfoColumns: string[] = [
    'id',
    'name',
    'description',
    'actionMenu',
  ];

  @ViewChildren(MatSort) sort = new QueryList<MatSort>();
  @ViewChildren(MatPaginator) paginator = new QueryList<MatPaginator>();

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

    this.projectFilterForm = formBuilder.group({
      valueString: '',
      activeProject: null,
    });
    this.projectFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLocaleLowerCase(),
        activeProject:
          value.activeProject === null || value.activeProject.length === 0
            ? null
            : value.activeProject[value.activeProject.length - 1] === 'true',
      } as string;
      this.projectsTableDataSource.filter = filter;
    });

    this.editCompanyProjectForm = this.formBuilder.group({
      name: ['', Validators.required],
      code: ['', Validators.required],
      active: [null, Validators.required],
    });

    this.activityFilterForm = formBuilder.group({
      valueString: '',
    });
    this.activityFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLocaleLowerCase(),
      } as string;
      this.activitiesTableDataSource.filter = filter;
    });

    this.userProjectAllocationViewFilteredUsers = of([]);
    this.userProjectAllocationViewFilteredProjects = of([]);
    this.activityProjectAssignmentViewFilteredProjects = of([]);
    this.activityProjectAssignmentViewFilteredActivities = of([]);
  }

  ngOnInit(): void {
    this.route.queryParamMap.subscribe((params) => {
      this.companyId = params.get('companyId');
      this.loadData();
    });
  }

  loadData() {
    this.loading = true;

    this.projectUnderEdit = null;

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
        projects: this.apiService.getCompanyProjects(this.companyId),
        activities: this.apiService.getCompanyProjectActivities(this.companyId),
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

          this.projects = response.projects;
          this.projectsTableDataSource = new MatTableDataSource(this.projects);
          setTimeout(() => {
            this.projectsTableDataSource.filterPredicate = (
              data,
              filter: any
            ) => {
              const activeProjectFilter =
                filter.activeProject === null
                  ? true
                  : data.active === filter.activeProject;
              const idFilter = data.id
                .toLocaleLowerCase()
                .includes(filter.valueString);
              const nameFilter = data.name
                .toLocaleLowerCase()
                .trim()
                .includes(filter.valueString);
              const codeFilter = data.code
                .toLocaleLowerCase()
                .trim()
                .includes(filter.valueString);

              return (
                (idFilter || nameFilter || codeFilter) && activeProjectFilter
              );
            };

            this.projectsTableDataSource.sort = this.sort.toArray()[2];
            this.projectsTableDataSource.paginator =
              this.paginator.toArray()[2];
          });

          this.userProjectAllocationViewFilteredProjects =
            this.projectUserAllocationViewForm.valueChanges.pipe(
              startWith(''),
              map((value: { project: string }) => {
                const name =
                  typeof value.project === 'string'
                    ? value.project
                    : value.project!;
                return name
                  ? this._filterProject(name as string)
                  : this.projects.slice();
              })
            );

          this.userProjectAllocationViewFilteredUsers =
            this.projectUserAllocationViewForm.valueChanges.pipe(
              startWith(''),
              map((value: { user: string }) => {
                const name =
                  typeof value.user === 'string' ? value.user : value.user!;
                return name
                  ? this._filterUsername(name as string)
                  : this.usersInCompany.slice();
              })
            );

          this.activities = response.activities;
          this.activitiesTableDataSource = new MatTableDataSource(
            this.activities
          );
          setTimeout(() => {
            this.activitiesTableDataSource.filterPredicate = (
              data,
              filter: any
            ) => {
              const idFilter = data.id
                .toLocaleLowerCase()
                .includes(filter.Validators);
              const nameFilter = data.name
                .toLocaleLowerCase()
                .includes(filter.valueString);
              return idFilter || nameFilter;
            };

            this.activitiesTableDataSource.sort = this.sort.toArray()[3];
            this.activitiesTableDataSource.paginator =
              this.paginator.toArray()[3];
          });

          this.activityProjectAssignmentViewFilteredProjects =
            this.activityProjectAllocationForm.valueChanges.pipe(
              startWith(''),
              map((value: { project: string }) => {
                const name =
                  typeof value.project === 'string'
                    ? value.project
                    : value.project!;
                return name
                  ? this._filterProject(name as string)
                  : this.projects.slice();
              })
            );

          this.activityProjectAssignmentViewFilteredActivities =
            this.activityProjectAllocationForm.valueChanges.pipe(
              startWith(''),
              map((value: { user: string }) => {
                const name =
                  typeof value.user === 'string' ? value.user : value.user!;
                return name
                  ? this._filterActivity(name as string)
                  : this.activities.slice();
              })
            );
        },
        error: () => {
          this.usersInCompany = [];
          this.usersTableDataSource = new MatTableDataSource(
            this.usersInCompany
          );
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

          this.projects = [];
          this.projectsTableDataSource = new MatTableDataSource(this.projects);
          setTimeout(() => {
            this.projectsTableDataSource.sort = this.sort.toArray()[2];
            this.projectsTableDataSource.paginator =
              this.paginator.toArray()[2];
          });
        },
      });
    }
  }

  private _filterProject(name: string): CompanyProjectInfo[] {
    const filterValue = name.toLowerCase();

    return this.projects.filter((option) =>
      option.name.toLowerCase().includes(filterValue)
    );
  }

  private _filterUsername(name: string): UserInCompanyInfo[] {
    const filterValue = name.toLowerCase();

    return this.usersInCompany.filter((option) =>
      option.userUsername.toLowerCase().includes(filterValue)
    );
  }

  private _filterActivity(name: string): ProjectActivityInfo[] {
    const filterValue = name.toLowerCase();

    return this.activities.filter((option) =>
      option.name.toLowerCase().includes(filterValue)
    );
  }

  onActiveProjectFilterChange(event: MatButtonToggleChange) {
    const toggle = event.source;
    if (toggle && event.value.some((item: string) => item === toggle.value)) {
      toggle.buttonToggleGroup.value = [toggle.value];
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

  openNewProjectDialog() {
    this.dialog
      .open(NewCompanyProjectDialogComponent, {
        width: '40rem',
        data: {
          companyId: this.companyId,
        },
      })
      .afterClosed()
      .subscribe({
        next: (data: NewCompanyProject) => {
          if (data !== undefined) {
            this.apiService
              .createCompanyProject(this.companyId!, data.name, data.code)
              .subscribe({
                next: () => {
                  this.loadData();
                  this.toastr.success('Project created', 'Success', {
                    timeOut: 5000,
                    progressBar: true,
                  });
                },
              });
          }
        },
      });
  }

  openNewActivityDialog() {
    this.dialog
      .open(NewActivityDialogComponent, {
        width: '40rem',
        data: {
          companyId: this.companyId,
        },
      })
      .afterClosed()
      .subscribe({
        next: (data: NewProjectActivity) => {
          if (data !== undefined) {
            this.apiService
              .createProjectActivity(
                this.companyId!,
                data.name,
                data.description
              )
              .subscribe({
                next: () => {
                  this.loadData();
                  this.toastr.success('Activity created', 'Success', {
                    timeOut: 5000,
                    progressBar: true,
                  });
                },
              });
          }
        },
      });
  }

  startEditActivity(activity: ProjectActivityInfo) {
    this.activityUnderEdit = activity.id;
    this.editActivityForm.setValue({
      name: activity.name,
      description: activity.description,
    });
  }

  confirmEditActivity(activity: ProjectActivityInfo) {
    this.apiService
      .editProjectActivity(
        this.companyId!,
        activity.id,
        this.editActivityForm.value['name'],
        this.editActivityForm.value['description']
      )
      .subscribe({
        next: () => {
          this.toastr.success(
            `Activity ${activity.name} updated`,
            'Activity updated',
            {
              timeOut: 5000,
              progressBar: true,
            }
          );
          this.activityUnderEdit = null;
          this.loadData();
        },
        error: () => {},
      });
  }

  cancelEditActivity(activity: ProjectActivityInfo) {
    this.activityUnderEdit = null;
    this.editActivityForm.setValue({
      name: '',
      description: '',
    });
  }

  deleteActivity(activity: ProjectActivityInfo) {
    this.apiService.deleteActivity(this.companyId!, activity.id).subscribe({
      next: () => {
        this.toastr.success(
          `Activity ${activity.name} deleted`,
          'Activity deleted',
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

  startEditProject(project: CompanyProjectInfo) {
    this.projectUnderEdit = project.id;
    this.editCompanyProjectForm.setValue({
      name: project.name,
      code: project.code,
      active: project.active,
    });
  }

  confirmEditProject(project: CompanyProjectInfo) {
    this.apiService
      .editCompanyProject(
        this.companyId!,
        project.id,
        this.editCompanyProjectForm.value['name'],
        this.editCompanyProjectForm.value['code'],
        this.editCompanyProjectForm.value['active'].toString() === 'true'
      )
      .subscribe({
        next: () => {
          this.toastr.success(
            `Project ${project.name} updated`,
            'Project updated',
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

  cancelEditProject(project: CompanyProjectInfo) {
    this.projectUnderEdit = null;
    this.editCompanyProjectForm.setValue({
      name: '',
      code: '',
      active: null,
    });
  }

  deleteProject(project: CompanyProjectInfo) {
    this.apiService
      .deleteCompanyProject(this.companyId!, project.id)
      .subscribe({
        next: () => {
          this.toastr.success(
            `Project ${project.name} deleted`,
            'Project deleted',
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

  showUserProjectAllocations() {
    if (
      this.userProjectAllocationViewMode ===
      UserProjectAllocationViewMode.PROJECT
    ) {
      this.currentAllocationUser = null;

      if (
        this.projectUserAllocationViewForm.value['project'] !== null &&
        this.projects.filter(
          (p) => p.name === this.projectUserAllocationViewForm.value['project']
        ).length != 0
      ) {
        this.currentAllocationProject =
          this.projectUserAllocationViewForm.value['project'];
        this.apiService
          .getCompanyProjectAllocationsByProject(
            this.companyId!,
            this.projects
              .filter((p) => p.name === this.currentAllocationProject!)
              .map((p) => p.id)[0]
          )
          .subscribe({
            next: (users: string[]) => {
              this.usersAllocatedInProject = users;
              this.userProjectAllocationModeShow = true;
              this.userProjectAllocationsForProjectForm.patchValue({
                usernames: this.usersInCompany
                  .filter((user) =>
                    this.usersAllocatedInProject.includes(user.userId)
                  )
                  .map((user) => user.userUsername), // Ensure it matches the values used in `mat-list-option`
              });
            },
            error: () => (this.userProjectAllocationModeShow = false),
          });
      }
    } else {
      this.currentAllocationProject = null;
      if (
        this.projectUserAllocationViewForm.value['username'] !== null &&
        this.usersInCompany.filter(
          (p) =>
            p.userUsername ===
            this.projectUserAllocationViewForm.value['username']
        ).length != 0
      ) {
        this.currentAllocationUser =
          this.projectUserAllocationViewForm.value['username'];
        this.apiService
          .getCompanyProjectAllocationsByUser(
            this.companyId!,
            this.usersInCompany
              .filter((u) => u.userUsername === this.currentAllocationUser!)
              .map((u) => u.userId)[0]
          )
          .subscribe({
            next: (projects: string[]) => {
              this.projectsAllocatedToUser = projects;
              this.userProjectAllocationModeShow = true;
              this.userProjectAllocationsForUserForm.patchValue({
                projects: this.projects
                  .filter((project) =>
                    this.projectsAllocatedToUser.includes(project.id)
                  )
                  .map((project) => project.name), // Ensure it matches the values used in `mat-list-option`
              });
            },
            error: () => (this.userProjectAllocationModeShow = false),
          });
      }
    }
  }

  startEditUserProjectAllocation() {
    this.userProjectAllocationModeUnderEdit = true;
  }

  cancelEditUserProjectAllocation() {
    this.userProjectAllocationModeUnderEdit = false;
    if (
      this.userProjectAllocationViewMode ===
      UserProjectAllocationViewMode.PROJECT
    ) {
      this.userProjectAllocationsForProjectForm.patchValue({
        usernames: this.usersInCompany
          .filter((user) => this.usersAllocatedInProject.includes(user.userId))
          .map((user) => user.userUsername), // Ensure it matches the values used in `mat-list-option`
      });
    } else {
      this.userProjectAllocationsForUserForm.patchValue({
        projects: this.projects
          .filter((project) =>
            this.projectsAllocatedToUser.includes(project.id)
          )
          .map((project) => project.name), // Ensure it matches the values used in `mat-list-option`
      });
    }
  }

  confirmEditUserProjectAllocation() {
    this.userProjectAllocationModeUnderEdit = false;
    if (
      this.userProjectAllocationViewMode ===
      UserProjectAllocationViewMode.PROJECT
    ) {
      this.apiService
        .updateCompanyProjectAllocationsByProject(
          this.companyId!,
          this.projects
            .filter((p) => p.name === this.currentAllocationProject!)
            .map((p) => p.id)[0],
          this.usersInCompany
            .filter((u) =>
              this.userProjectAllocationsForProjectForm.value[
                'usernames'
              ]!.includes(u.userUsername)
            )
            .map((u) => u.userId)
        )
        .subscribe({
          next: () => {
            this.toastr.success(
              `Project allocations ${this.currentAllocationProject} updated`,
              'Update succeeded',
              {
                timeOut: 5000,
                progressBar: true,
              }
            );
          },
          error: () => {},
        });
    } else {
      this.apiService
        .updateCompanyProjectAllocationsByUser(
          this.companyId!,
          this.usersInCompany
            .filter((u) => u.userUsername === this.currentAllocationUser!)
            .map((u) => u.userId)[0],
          this.projects
            .filter((p) =>
              this.userProjectAllocationsForUserForm.value[
                'projects'
              ]!.includes(p.name)
            )
            .map((p) => p.id)
        )
        .subscribe({
          next: () => {
            this.toastr.success(
              `User allocations ${this.currentAllocationUser} updated`,
              'Update succeeded',
              {
                timeOut: 5000,
                progressBar: true,
              }
            );
          },
          error: () => {},
        });
    }
  }

  showActivityProjectAllocations() {
    if (
      this.activityProjectAllocationViewMode ===
      ActivityProjectAllocationViewMode.PROJECT
    ) {
      this.currentAssignmentActivity = null;

      if (
        this.activityProjectAllocationForm.value['project'] !== null &&
        this.projects.filter(
          (p) => p.name === this.activityProjectAllocationForm.value['project']
        ).length != 0
      ) {
        this.currentAssignmentProject =
          this.activityProjectAllocationForm.value['project'];
        this.apiService
          .getCompanyProjectActivitiesByProject(
            this.companyId!,
            this.projects
              .filter((p) => p.name === this.currentAssignmentProject!)
              .map((p) => p.id)[0]
          )
          .subscribe({
            next: (users: string[]) => {
              this.activitiesAssignedInProject = users;
              this.activityProjectAssignmentModeShow = true;
              this.assignmentsForProjectForm.patchValue({
                activities: this.activities
                  .filter((activity) =>
                    this.activitiesAssignedInProject.includes(activity.id)
                  )
                  .map((activity) => activity.name), // Ensure it matches the values used in `mat-list-option`
              });
            },
            error: () => (this.activityProjectAssignmentModeShow = false),
          });
      }
    } else {
      this.currentAssignmentProject = null;
      if (
        this.activityProjectAllocationForm.value['activity'] !== null &&
        this.activities.filter(
          (p) => p.name === this.activityProjectAllocationForm.value['activity']
        ).length != 0
      ) {
        this.currentAssignmentActivity =
          this.activityProjectAllocationForm.value['activity'];
        this.apiService
          .getCompanyProjectActivitiesByActivity(
            this.companyId!,
            this.activities
              .filter((u) => u.name === this.currentAssignmentActivity!)
              .map((u) => u.id)[0]
          )
          .subscribe({
            next: (projects: string[]) => {
              this.projectsAssignedToActivity = projects;
              this.activityProjectAssignmentModeShow = true;
              this.assignmentsForActivityForm.patchValue({
                projects: this.projects
                  .filter((project) =>
                    this.projectsAssignedToActivity.includes(project.id)
                  )
                  .map((project) => project.name), // Ensure it matches the values used in `mat-list-option`
              });
            },
            error: () => (this.activityProjectAssignmentModeShow = false),
          });
      }
    }
  }

  startEditActivityProjectAllocation() {
    this.activityProjectAssignmentModeUnderEdit = true;
  }

  cancelEditActivityProjectAllocation() {
    this.activityProjectAssignmentModeUnderEdit = false;
    if (
      this.activityProjectAllocationViewMode ===
      ActivityProjectAllocationViewMode.PROJECT
    ) {
      this.assignmentsForProjectForm.patchValue({
        activities: this.activities
          .filter((activity) =>
            this.activitiesAssignedInProject.includes(activity.id)
          )
          .map((activity) => activity.name),
      });
    } else {
      this.assignmentsForActivityForm.patchValue({
        projects: this.projects
          .filter((project) =>
            this.projectsAssignedToActivity.includes(project.id)
          )
          .map((project) => project.name),
      });
    }
  }

  confirmEditActivityProjectAllocation() {
    this.activityProjectAssignmentModeUnderEdit = false;
    if (
      this.activityProjectAllocationViewMode ===
      ActivityProjectAllocationViewMode.PROJECT
    ) {
      this.apiService
        .updateCompanyProjectActivitiesByProject(
          this.companyId!,
          this.projects
            .filter((p) => p.name === this.currentAssignmentProject!)
            .map((p) => p.id)[0],
          this.activities
            .filter((u) =>
              this.assignmentsForProjectForm.value['activities']!.includes(
                u.name
              )
            )
            .map((u) => u.id)
        )
        .subscribe({
          next: () => {
            this.toastr.success(
              `Project activities assignment ${this.currentAssignmentProject} updated`,
              'Update succeeded',
              {
                timeOut: 5000,
                progressBar: true,
              }
            );
          },
          error: () => {},
        });
    } else {
      this.apiService
        .updateCompanyProjectActivitiesByActivity(
          this.companyId!,
          this.activities
            .filter((u) => u.name === this.currentAssignmentActivity!)
            .map((u) => u.id)[0],
          this.projects
            .filter((p) =>
              this.assignmentsForActivityForm.value['projects']!.includes(
                p.name
              )
            )
            .map((p) => p.id)
        )
        .subscribe({
          next: () => {
            this.toastr.success(
              `Activity project assignment ${this.currentAssignmentActivity} updated`,
              'Update succeeded',
              {
                timeOut: 5000,
                progressBar: true,
              }
            );
          },
          error: () => {},
        });
    }
  }
}
