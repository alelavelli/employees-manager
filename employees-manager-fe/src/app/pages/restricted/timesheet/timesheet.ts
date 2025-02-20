import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { CalendarDay, CompanyInfo, TimesheetDay } from '../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import { ApiService } from '../../../service/api.service';
import { Day, Month } from '../../../types/enums';
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
  loading: boolean = false;

  companyId: string | null = null;
  companies: CompanyInfo[] = [];
  selectedMonth: Month | null = null;
  selectedYear: number | null = null;
  openTimesheet: boolean = false;

  years: number[] = [...Array(30).keys()].map((i) => i + 2025);
  months: Month[] = [
    Month.January,
    Month.February,
    Month.March,
    Month.April,
    Month.May,
    Month.June,
    Month.July,
    Month.August,
    Month.September,
    Month.October,
    Month.November,
    Month.December,
  ];
  calendarDays: CalendarDay[] = [];
  userDays: TimesheetDay[] = [];

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {}

  loadData() {
    if (this.selectedYear !== null && this.selectedMonth !== null) {
      const days = new Date(this.selectedYear, this.selectedMonth, 0).getDate();
      this.calendarDays = [...Array(days).keys()].map((i) => {
        const day = i + 1;
        const weekDay = new Date(
          this.selectedYear!,
          this.selectedMonth!,
          day
        ).getDay();
        const dayName = Day[weekDay];
        return {
          dayName: dayName,
          dayNumber: day,
          isWeekend: weekDay == 0 || weekDay == 6,
        };
      });

      this.openTimesheet = true;
    }
  }
}
