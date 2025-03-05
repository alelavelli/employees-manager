import { Component, OnInit, QueryList, ViewChildren } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { CorporateGroupInfo } from '../../../types/model';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { FormBuilder, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { ApiService } from '../../../service/api.service';
import { ActivatedRoute } from '@angular/router';
import { MatSelectModule } from '@angular/material/select';
import { forkJoin } from 'rxjs';
import { MatSort, MatSortModule } from '@angular/material/sort';
import { MatPaginator, MatPaginatorModule } from '@angular/material/paginator';
import { MatInputModule } from '@angular/material/input';
import { MatMenuModule } from '@angular/material/menu';
import { MatDialog } from '@angular/material/dialog';
import { ConfirmDialogComponent } from '../../../components/confirm-modal/confirm-modal';
import { ToastrService } from 'ngx-toastr';
import { EditCorporateGroupDialogComponent } from './edit-corporate-group-modal/edit-corporate-group-modal';

@Component({
  selector: 'corporate-group-page',
  templateUrl: './corporate-group.html',
  styleUrls: ['./corporate-group.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatSelectModule,
    MatProgressBarModule,
    MatFormFieldModule,
    MatIconModule,
    MatButtonModule,
    MatTableModule,
    ReactiveFormsModule,
    MatSortModule,
    MatPaginatorModule,
    MatInputModule,
    MatMenuModule,
  ],
})
export class CorporateGroupPageComponent implements OnInit {
  loading: boolean = false;
  corporateGroupId: string | null = null;
  corporateGroup: CorporateGroupInfo | null = null;
  corporateGroups: CorporateGroupInfo[] = [];

  companies: string[] = [];
  companiesTableDataSource: MatTableDataSource<string> =
    new MatTableDataSource<string>([]);
  readonly companiesFilterForm: FormGroup;
  displayedCompaniesInfoColumns: string[] = ['name', 'actionMenu'];
  @ViewChildren(MatSort) sort = new QueryList<MatSort>();
  @ViewChildren(MatPaginator) paginator = new QueryList<MatPaginator>();

  constructor(
    private route: ActivatedRoute,
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    private dialog: MatDialog,
    private toastr: ToastrService
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
    this.loading = true;
    this.loadCorporateGroups();
  }

  loadCorporateGroups() {
    forkJoin({
      corporateGroups: this.apiService.getUserCorporateGroups(),
    }).subscribe({
      next: (response) => {
        this.corporateGroups = response.corporateGroups;
        this.route.queryParamMap.subscribe((params) => {
          this.corporateGroupId = params.get('corporateGroupId');
          this.loadData();
        });
        this.loading = false;
      },
      error: () => {
        this.loading = false;
      },
    });
  }

  loadData() {
    if (this.corporateGroupId !== null) {
      this.corporateGroup = this.corporateGroups.filter(
        (group) => group.groupId === this.corporateGroupId
      )[0];

      if (this.corporateGroup !== null && this.corporateGroup !== undefined) {
        this.loading = true;
        this.companies = this.corporateGroup.companyNames;
        this.companiesTableDataSource = new MatTableDataSource(this.companies);
        this.companiesTableDataSource.filterPredicate = (
          data,
          filter: string
        ) => {
          return data.toLocaleLowerCase().includes(filter);
        };
        this.companiesTableDataSource.sort = this.sort.toArray()[0];
        this.companiesTableDataSource.paginator = this.paginator.toArray()[0];
        this.loading = false;
      }
    }
  }

  editCorporateGroupDialog() {
    if (this.corporateGroup != null) {
      this.dialog
        .open(EditCorporateGroupDialogComponent, {
          data: {
            corporateGroup: this.corporateGroup,
          },
        })
        .afterClosed()
        .subscribe({
          next: (update: CorporateGroupInfo | undefined) => {
            if (update !== undefined) {
              this.apiService
                .editCorporateGroup(this.corporateGroupId!, update)
                .subscribe({
                  next: () => {
                    this.loadCorporateGroups();
                    this.loadData();
                    this.toastr.success(
                      'Update complete',
                      'Corporate Group updated correctly',
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

  deleteCorporateGroupDialog() {
    if (this.corporateGroupId != null) {
      this.dialog
        .open(ConfirmDialogComponent, {
          data: {
            title: 'Confirm delete',
            content: `Are you sure to delete Corporate Group ${this.corporateGroup?.name}?`,
          },
        })
        .afterClosed()
        .subscribe({
          next: (confirm) => {
            if (confirm) {
              this.apiService
                .deleteCorporateGroup(this.corporateGroupId!)
                .subscribe({
                  next: () => {
                    this.toastr.success(
                      'Delete complete',
                      'Corporate group deleted',
                      {
                        timeOut: 5000,
                        progressBar: true,
                      }
                    );
                    this.corporateGroupId = null;
                    this.corporateGroup = null;
                    this.companies = [];
                    this.corporateGroups = [];
                    this.companiesTableDataSource = new MatTableDataSource(
                      this.companies
                    );
                    this.loadData();
                  },
                });
            }
          },
        });
    }
  }
}
