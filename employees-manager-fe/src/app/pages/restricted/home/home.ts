import { Component, OnInit, ViewChild } from '@angular/core';
import { CompanyInfo, UserData } from '../../../types/model';
import { UserService } from '../../../service/user.service';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';
import { FormBuilder, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
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
  ],
})
export class HomePageComponent implements OnInit {
  loading = false;
  userData: UserData | null = null;

  companies: CompanyInfo[] = [];
  companiesTableDataSource: MatTableDataSource<CompanyInfo> =
    new MatTableDataSource<CompanyInfo>([]);
  readonly companiesFilterForm: FormGroup;
  displayedCompaniesInfoColumns: string[] = ['id', 'name', 'actionMenu'];

  @ViewChild(MatSort, { static: false }) sort!: MatSort;
  @ViewChild(MatPaginator, { static: false }) paginator!: MatPaginator;

  constructor(
    private userService: UserService,
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    private toastr: ToastrService,
    private dialog: MatDialog
  ) {
    this.companiesFilterForm = formBuilder.group({
      valueString: '',
    });
    this.companiesFilterForm.valueChanges.subscribe((value) => {
      const filter = {
        ...value,
        valueString: value.valueString.trim().toLowerCase(),
      } as string;
      this.companiesTableDataSource.filter = filter;
    });
  }

  ngOnInit(): void {
    this.loadData();
  }

  loadData() {
    this.loading = false;

    forkJoin({
      userData: this.userService.fetchUserData(),
      companies: this.apiService.getUserCompanies(),
    }).subscribe({
      next: (response) => {
        this.userData = response.userData;
        this.companies = response.companies;
        this.companiesTableDataSource = new MatTableDataSource(this.companies);
        setTimeout(() => {
          this.companiesTableDataSource.filterPredicate = (
            data,
            filter: any
          ) => {
            const nameFilter = data.name
              .toLocaleLowerCase()
              .includes(filter.valueString);
            return nameFilter;
          };
          this.companiesTableDataSource.sort = this.sort;
          this.companiesTableDataSource.paginator = this.paginator;
        });
        this.loading = false;
      },
      error: () => {
        this.userData = null;
        this.companies = [];
        this.companiesTableDataSource = new MatTableDataSource(this.companies);
        setTimeout(() => {
          this.companiesTableDataSource.sort = this.sort;
          this.companiesTableDataSource.paginator = this.paginator;
        });
        this.loading = false;
      },
    });
  }

  openCreateCompanyDialog() {}
}
