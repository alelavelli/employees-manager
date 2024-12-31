import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewChild, ViewEncapsulation } from '@angular/core';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { ApiService } from '../../../service/api.service';
import { AdminPanelOverview, AdminPanelUserInfo } from '../../../types/model';
import { forkJoin } from 'rxjs';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { MatIconModule } from '@angular/material/icon';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import {
  MatButtonToggleChange,
  MatButtonToggleModule,
} from '@angular/material/button-toggle';
import {
  AbstractControl,
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
} from '@angular/forms';

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
    private formBuilder: FormBuilder
  ) {
    this.userFilterForm = formBuilder.group({
      valueString: '',
      activeUser: null,
    });
    this.userFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLowerCase(),
        activeUser:
          value.activeUser === null || value.activeUser.length === 0
            ? null
            : value.activeUser[value.activeUser.length - 1] === 'true',
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

            const idFilter = data.id.toLocaleLowerCase().includes(filter);
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

  openUserActionsMenu(element: any) {
    console.log('Open user actions menu for user:', element);
  }
}
