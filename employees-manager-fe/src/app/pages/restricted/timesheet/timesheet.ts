import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import {
  CalendarDay,
  TimesheetActivityHours,
  TimesheetDay,
  TimesheetProjectInfo,
  UserData,
} from '../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import { ApiService } from '../../../service/api.service';
import { Day, Month, TimesheetDayWorkType } from '../../../types/enums';
import { UserService } from '../../../service/user.service';
import moment from 'moment';
import { ToastrService } from 'ngx-toastr';
import { MatDialog } from '@angular/material/dialog';
import { EditTimesheetDialogComponent } from './timesheet-day-modal/timesheet-day-modal';
import { timesheetDayWorkTypeToString } from '../../../service/common';
import { MatButtonModule } from '@angular/material/button';
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
    MatButtonModule,
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

  offsetDays: CalendarDay[] = [];

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
      const days = moment(
        `${this.selectedYear}-${this.selectedMonth}`,
        'YYYY-M'
      ).daysInMonth();
      this.calendarDays = [...Array(days).keys()].map((i) => {
        const day = i + 1;
        const date = moment.utc(
          `${this.selectedYear}-${this.selectedMonth}-${day}`,
          'YYYY-M-D'
        );
        const weekDay = date.day();
        const dayName = Day[weekDay];
        return {
          date: date,
          dayName: dayName,
          dayNumber: day,
          isWeekend: weekDay == 0 || weekDay == 6,
        };
      });
      this.offsetDays = [...Array(this.calendarDays[0].date.day() - 1)];

      this.apiService
        .getTimesheetDays(
          this.userData.id,
          this.selectedYear,
          this.selectedMonth
        )
        .subscribe({
          next: (value) => {
            this.userDays = value.map((obj) => ({
              ...obj,
              // Convert date to UTC Moment because it is not done automatically. Nice language :)
              date: moment.utc(obj.date, 'YYYY-MM-DD HH:mm:ss'),
            }));
          },
        });

      this.openTimesheet = true;
    }
  }

  getMatchingUserDay(day: CalendarDay): TimesheetDay | null {
    if (this.selectedMonth !== null && this.selectedYear !== null) {
      const filtered = this.userDays.filter((timesheetDay) => {
        return (
          timesheetDay.date.date() === day.dayNumber &&
          timesheetDay.date.month() + 1 === this.selectedMonth &&
          timesheetDay.date.year() === this.selectedYear
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
      return timesheetDayWorkTypeToString(matchingTimesheetDay.workingType);
    }
  }

  getTotalWorkHours(day: CalendarDay): number | null {
    let matchingTimesheetDay = this.getMatchingUserDay(day);
    if (matchingTimesheetDay === null) {
      return null;
    } else if (matchingTimesheetDay.activities.length > 0) {
      return matchingTimesheetDay.activities
        .map((elem) => elem.hours)
        .reduce((tot, elem) => tot + elem);
    } else {
      return 0;
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
        .subscribe({
          next: (data: {
            permitHours: number;
            dayType: TimesheetDayWorkType;
            activities: TimesheetActivityHours[];
          }) => {
            if (data !== undefined && this.userData !== null) {
              this.apiService
                .createTimesheetDay(
                  this.userData.id,
                  day.date,
                  data.permitHours,
                  data.dayType,
                  data.activities
                )
                .subscribe({
                  next: () => {
                    this.loadData();
                    this.toastr.success('Updated timesheet day', 'Sent', {
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

  exportTimesheet() {
    this.apiService
      .exportUserTimesheet(this.selectedYear!, this.selectedMonth!)
      .subscribe({
        next: (response) => {
          const blob = new Blob([response], {
            type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
          });
          const url = window.URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `timesheet ${this.selectedYear}-${this.selectedMonth}.xlsx`;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          window.URL.revokeObjectURL(url);
        },
        error: () => {},
      });
  }
}
