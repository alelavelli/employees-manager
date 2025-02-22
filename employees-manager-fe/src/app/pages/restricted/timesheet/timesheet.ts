import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import {
  CalendarDay,
  TimesheetDay,
  TimesheetProjectInfo,
  UserData,
} from '../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import { ApiService } from '../../../service/api.service';
import { Day, Month, TimesheetDayWorkType } from '../../../types/enums';
import { UserService } from '../../../service/user.service';
import { M } from '@angular/cdk/keycodes';
import { ToastrService } from 'ngx-toastr';
import { MatDialog } from '@angular/material/dialog';
import { EditTimesheetDialogComponent } from './timesheet-day-modal/timesheet-day-modal';
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

  userData: UserData | null = null;
  userProjects: TimesheetProjectInfo[] = [];

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

  constructor(
    private apiService: ApiService,
    private userService: UserService,
    private toastr: ToastrService,
    private dialog: MatDialog
  ) {}

  ngOnInit(): void {
    this.userService.fetchUserData().subscribe({
      next: (value) => {
        this.userData = value;
        this.apiService
          .getUserProjectsForTimesheet(this.userData.id)
          .subscribe({
            next: (projectResponse) => {
              this.userProjects = projectResponse;
            },
          });
      },
    });
  }

  loadData() {
    if (
      this.selectedYear !== null &&
      this.selectedMonth !== null &&
      this.userData !== null
    ) {
      const days = new Date(this.selectedYear, this.selectedMonth, 0).getDate();
      this.calendarDays = [...Array(days).keys()].map((i) => {
        const day = i + 1;
        const date = new Date(this.selectedYear!, this.selectedMonth!, day);
        const weekDay = new Date(
          this.selectedYear!,
          this.selectedMonth!,
          day
        ).getDay();
        const dayName = Day[weekDay];
        return {
          date: date,
          dayName: dayName,
          dayNumber: day,
          isWeekend: weekDay == 0 || weekDay == 6,
        };
      });

      this.apiService
        .getTimesheetDays(
          this.userData.id,
          this.selectedYear,
          this.selectedMonth
        )
        .subscribe({
          next: (value) => {
            this.userDays = value;
          },
        });

      this.openTimesheet = true;
    }
  }

  getMatchingUserDay(day: CalendarDay): TimesheetDay | null {
    if (this.selectedMonth !== null && this.selectedYear !== null) {
      const filtered = this.userDays.filter((timesheetDay) => {
        return (
          timesheetDay.date.getDate() === day.dayNumber &&
          timesheetDay.date.getMonth() + 1 === this.selectedMonth &&
          timesheetDay.date.getFullYear() === this.selectedYear
        );
      });
      if (filtered.length > 0) {
        return filtered[0];
      } else {
        return null;
      }
    } else {
      return null;
    }
  }

  getWorkType(day: CalendarDay): string | null {
    let matchingTimesheetDay = this.getMatchingUserDay(day);
    if (matchingTimesheetDay === null) {
      return null;
    } else {
      return matchingTimesheetDay.workingType;
    }
  }

  getTotalWorkHours(day: CalendarDay): number | null {
    let matchingTimesheetDay = this.getMatchingUserDay(day);
    if (matchingTimesheetDay === null) {
      return null;
    } else {
      return matchingTimesheetDay.activities
        .map((elem) => elem.hours)
        .reduce((tot, elem) => tot + elem);
    }
  }

  getPermitHours(day: CalendarDay): number | null {
    let matchingTimesheetDay = this.getMatchingUserDay(day);
    if (matchingTimesheetDay === null) {
      return null;
    } else {
      return matchingTimesheetDay.permitHours;
    }
  }

  getNumberOfProjects(day: CalendarDay): number | null {
    let matchingTimesheetDay = this.getMatchingUserDay(day);
    if (matchingTimesheetDay === null) {
      return null;
    } else {
      return new Set(
        matchingTimesheetDay.activities.map((elem) => elem.projectId)
      ).size;
    }
  }

  openEditTimesheetDay(day: CalendarDay) {
    if (!day.isWeekend) {
      this.dialog
        .open(EditTimesheetDialogComponent, {
          data: {
            calendarDay: day,
            timesheetDay: this.getMatchingUserDay(day),
            projects: this.userProjects,
          },
        })
        .afterClosed()
        .subscribe({});
    }
  }
}
