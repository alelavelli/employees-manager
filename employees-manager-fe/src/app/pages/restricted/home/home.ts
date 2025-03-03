import { Component, OnInit, QueryList, ViewChildren } from '@angular/core';
import {
  CompanyInfo,
  CorporateGroupInfo,
  CreateCompanyParameters,
  CreateCorporateGroupParameters,
  UserData,
} from '../../../types/model';
import { UserService } from '../../../service/user.service';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';
import { FormBuilder, FormGroup, ReactiveFormsModule } from '@angular/forms';
import {
  MatButtonToggleChange,
  MatButtonToggleModule,
} from '@angular/material/button-toggle';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatMenuModule } from '@angular/material/menu';
import { ApiService } from '../../../service/api.service';
import { ToastrService } from 'ngx-toastr';
import { MatDialog } from '@angular/material/dialog';
import { forkJoin } from 'rxjs';
import { CompanyRole } from '../../../types/enums';
import { NewCompanyDialogComponent } from './new-company-modal/new-company-modal';
import { RouterLink } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { NewCorporateGroupDialogComponent } from './new-corporate-group-modal/new-corporate-group-modal';

@Component({
  selector: 'home-page',
  templateUrl: './home.html',
  styleUrls: ['./home.scss'],
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
    RouterLink,
    MatButtonModule,
  ],
})
export class HomePageComponent implements OnInit {
  CompanyRole = CompanyRole;

  loading: boolean = false;
  userData: UserData | null = null;

  companies: CompanyInfo[] = [];
  companiesTableDataSource: MatTableDataSource<CompanyInfo> =
    new MatTableDataSource<CompanyInfo>([]);
  readonly companiesFilterForm: FormGroup;
  displayedCompaniesInfoColumns: string[] = [
    'id',
    'name',
    'active',
    'totalUsers',
    'actionMenu',
  ];

  corporateGroups: CorporateGroupInfo[] = [];
  corporateGroupsTableDataSource: MatTableDataSource<CorporateGroupInfo> =
    new MatTableDataSource<CorporateGroupInfo>([]);
  readonly corporateGroupsFilterForm: FormGroup;
  displayedCorporateGroupInfoColumns: string[] = [
    'id',
    'name',
    'totalCompanies',
    'actionMenu',
  ];

  @ViewChildren(MatSort) sort = new QueryList<MatSort>();
  @ViewChildren(MatPaginator) paginator = new QueryList<MatPaginator>();

  constructor(
    private userService: UserService,
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    private toastr: ToastrService,
    private dialog: MatDialog
  ) {
    this.companiesFilterForm = formBuilder.group({
      valueString: '',
      activeCompany: null,
    });
    this.companiesFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLowerCase(),
        activeCompany:
          value.activeCompany === null || value.activeCompany.length === 0
            ? null
            : value.activeCompany[value.activeCompany.length - 1] === 'true',
      } as string;
      this.companiesTableDataSource.filter = filter;
    });

    this.corporateGroupsFilterForm = formBuilder.group({
      valueString: '',
    });
    this.corporateGroupsFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLowerCase(),
      } as string;
      this.corporateGroupsTableDataSource.filter = filter;
    });
  }

  ngOnInit(): void {
    this.loadData();
  }

  loadData() {
    this.loading = true;

    forkJoin({
      userData: this.userService.fetchUserData(),
      companies: this.apiService.getUserCompanies(),
      corporateGroups: this.apiService.getUserCorporateGroups(),
    }).subscribe({
      next: (response) => {
        this.userData = response.userData;
        this.companies = response.companies;
        this.corporateGroups = response.corporateGroups;

        this.companiesTableDataSource = new MatTableDataSource(this.companies);
        this.corporateGroupsTableDataSource = new MatTableDataSource(
          this.corporateGroups
        );
        setTimeout(() => {
          this.companiesTableDataSource.filterPredicate = (
            data,
            filter: any
          ) => {
            const activeCompanyFilter =
              filter.activeCompany === null
                ? true
                : data.active === filter.activeCompany;
            const idFilter = data.id
              .toLocaleLowerCase()
              .includes(filter.valueString);
            const nameFilter = data.name
              .toLocaleLowerCase()
              .includes(filter.valueString);
            return activeCompanyFilter && (nameFilter || idFilter);
          };
          this.companiesTableDataSource.sort = this.sort.toArray()[0];
          this.companiesTableDataSource.paginator = this.paginator.toArray()[0];

          this.corporateGroupsTableDataSource.filterPredicate = (
            data,
            filter: any
          ) => {
            const idFilter = data.groupId
              .toLocaleLowerCase()
              .includes(filter.valueString);
            const nameFilter = data.name
              .toLocaleLowerCase()
              .includes(filter.valueString);
            return nameFilter || idFilter;
          };
          this.corporateGroupsTableDataSource.sort = this.sort.toArray()[1];
          this.corporateGroupsTableDataSource.paginator =
            this.paginator.toArray()[1];
        });
        this.loading = false;
      },
      error: () => {
        this.userData = null;
        this.companies = [];
        this.corporateGroups = [];
        this.companiesTableDataSource = new MatTableDataSource(this.companies);
        this.corporateGroupsTableDataSource = new MatTableDataSource(
          this.corporateGroups
        );
        setTimeout(() => {
          this.companiesTableDataSource.sort = this.sort.toArray()[0];
          this.companiesTableDataSource.paginator = this.paginator.toArray()[0];
          this.corporateGroupsTableDataSource.sort = this.sort.toArray()[1];
          this.corporateGroupsTableDataSource.paginator =
            this.paginator.toArray()[1];
        });
        this.loading = false;
      },
    });
  }

  onActiveCompanyFilterChange(event: MatButtonToggleChange) {
    const toggle = event.source;
    if (toggle && event.value.some((item: string) => item === toggle.value)) {
      toggle.buttonToggleGroup.value = [toggle.value];
    }
  }

  openCreateCompanyDialog() {
    this.dialog
      .open(NewCompanyDialogComponent, {
        width: '40rem',
        data: {},
      })
      .afterClosed()
      .subscribe({
        next: (newCompany: CreateCompanyParameters | undefined) => {
          if (newCompany !== undefined) {
            this.apiService.createCompany(newCompany).subscribe({
              next: (companyId: string) => {
                this.loadData();
                this.toastr.success(
                  'New company created with id ' + companyId,
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
}
