import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { CompanyInfo, TimesheetDay } from '../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import { forkJoin } from 'rxjs';
import { ApiService } from '../../../service/api.service';
@Component({
  selector: 'timesheet-page',
  templateUrl: './timesheet.html',
  styleUrls: ['./timesheet.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatProgressBarModule,
    MatFormFieldModule,
    MatSelectModule,
    MatIconModule,
  ],
})
export class TimesheetPageComponent implements OnInit {
  values = [...Array(35).keys()];

  loading: boolean = false;

  companyId: string | null = null;
  companies: CompanyInfo[] = [];
  selectedMonth: string | null = null;
  months: string[] = [];

  days: TimesheetDay[] = [];

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.loadData();
  }

  loadData() {
    //this.loading = true;
    //this.apiService.getUserCompanies();
  }
}
