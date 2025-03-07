import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute } from '@angular/router';
import { forkJoin } from 'rxjs';
import { UserService } from '../../../service/user.service';
import { CompanyInfo, UserData, UserInCompanyInfo } from '../../../types/model';
import { MatSelectModule } from '@angular/material/select';

import { ApiService } from '../../../service/api.service';
import { CompanyRole } from '../../../types/enums';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatIconModule } from '@angular/material/icon';

import { ToastrService } from 'ngx-toastr';
import { MatDialog } from '@angular/material/dialog';
import { MatTabsModule } from '@angular/material/tabs';

import { MatButtonModule } from '@angular/material/button';
import { CompanyUsers } from './users/users';
import { CompanyProjects } from './projects/projects';

@Component({
  selector: 'company-page',
  templateUrl: './company.html',
  styleUrls: ['./company.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatSelectModule,
    MatProgressBarModule,
    MatIconModule,
    MatTabsModule,
    MatButtonModule,
    CompanyUsers,
    CompanyProjects,
  ],
})
export class CompanyPageComponent implements OnInit {
  loading: boolean = false;
  userData: UserData | null = null;
  companyId: string | null = null;
  selectedCompany: CompanyInfo | null = null;
  companies: CompanyInfo[] = [];

  usersInCompany: UserInCompanyInfo[] = [];

  constructor(
    private route: ActivatedRoute,
    private userService: UserService,
    private apiService: ApiService,
    private toastr: ToastrService,
    private dialog: MatDialog
  ) {}

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
        this.selectedCompany = this.companies.filter(
          (company) => company.id == this.companyId
        )[0];

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
      }).subscribe({
        next: (response) => {
          this.usersInCompany = response.users;
        },
        error: () => {
          this.usersInCompany = [];
        },
      });
    }
  }
}
