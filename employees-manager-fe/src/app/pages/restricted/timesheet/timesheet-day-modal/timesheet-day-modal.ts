import { CommonModule } from '@angular/common';
import { Component, Inject, ViewEncapsulation } from '@angular/core';
import {
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import {
  MAT_DIALOG_DATA,
  MatDialogModule,
  MatDialogRef,
} from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import {
  CalendarDay,
  TimesheetActivityHours,
  TimesheetDay,
  TimesheetProjectInfo,
} from '../../../../types/model';
import { TimesheetDayWorkType } from '../../../../types/enums';
import { MatSelectModule } from '@angular/material/select';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { MatMenuModule } from '@angular/material/menu';
import { MatTooltipModule } from '@angular/material/tooltip';

interface Company {
  name: string;
  id: string;
}

interface Project {
  name: string;
  id: string;
}

interface Activity {
  name: string;
  id: string;
}

@Component({
  selector: 'timesheet-day-modal',
  templateUrl: './timesheet-day-modal.html',
  styleUrls: ['./timesheet-day-modal.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatIconModule,
    MatButtonModule,
    MatInputModule,
    MatDialogModule,
    MatFormFieldModule,
    ReactiveFormsModule,
    MatSelectModule,
    MatTableModule,
    MatMenuModule,
    MatTooltipModule,
  ],
  encapsulation: ViewEncapsulation.None,
})
export class EditTimesheetDialogComponent {
  TimesheetDayWorkType = TimesheetDayWorkType;

  activities: TimesheetActivityHours[];
  activitiesTableDataSource: MatTableDataSource<TimesheetActivityHours> =
    new MatTableDataSource<TimesheetActivityHours>([]);
  displayedActivitiesColumns: string[] = [
    'companyName',
    'projectName',
    'activityName',
    'hours',
    'notes',
    'actionMenu',
  ];

  dayForm: FormGroup = this.formBuilder.group({
    dayType: ['', Validators.required],
    permitHours: [
      0,
      [Validators.required, Validators.min(0), Validators.max(4)],
    ],
  });

  editActivityForm: FormGroup = this.formBuilder.group({
    companyId: ['', Validators.required],
    projectId: ['', Validators.required],
    activityId: ['', Validators.required],
    hours: [0, Validators.required],
    notes: [''],
  });
  activityUnderEdit: string | null = null;

  calendarDay: CalendarDay;
  timesheetDay: TimesheetDay | null;
  projects: TimesheetProjectInfo[];

  companies: Company[];
  projectsOfCompany: Project[];
  activitiesOfProject: Activity[];

  constructor(
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<TimesheetDay>,
    @Inject(MAT_DIALOG_DATA)
    public data: {
      calendarDay: CalendarDay;
      timesheetDay: TimesheetDay | null;
      projects: TimesheetProjectInfo[];
    }
  ) {
    this.calendarDay = data.calendarDay;
    this.timesheetDay = data.timesheetDay;
    this.projects = data.projects;
    this.companies = this.getCompanies();
    this.projectsOfCompany = [];
    this.activitiesOfProject = [];
    this.activities =
      data.timesheetDay === null ? [] : data.timesheetDay.activities;
    this.activitiesTableDataSource = new MatTableDataSource(this.activities);
    if (this.timesheetDay !== null) {
      this.dayForm.setValue({
        dayType: this.timesheetDay.workingType,
        permitHours: this.timesheetDay.permitHours,
      });
    }
  }

  hoursLessThanNine(): boolean {
    return (
      this.activities
        .map((elem) => elem.hours)
        .reduce((sum, current) => sum + current, 0) +
        this.dayForm.value['permitHours'] <=
      8
    );
  }

  getCompanies(): Company[] {
    return Array.from(
      new Map(
        this.projects.map((elem) => [
          elem.companyId,
          { name: elem.companyName, id: elem.companyId },
        ])
      ).values()
    );
  }

  updateProjectsOfCompany() {
    if (this.editActivityForm.value['companyId'] !== null) {
      this.projectsOfCompany = Array.from(
        new Map(
          this.projects
            .filter(
              (elem) =>
                elem.companyId === this.editActivityForm.value['companyId']
            )
            .map((elem) => [
              elem.projectId,
              { name: elem.projectName, id: elem.projectId },
            ])
        ).values()
      );
      this.updateActivitiesOfProject();
    }
  }

  updateActivitiesOfProject() {
    if (this.editActivityForm.value['projectId'] !== null) {
      var project = this.projects.filter(
        (elem) => elem.projectId === this.editActivityForm.value['projectId']
      )[0];

      this.activitiesOfProject = project.activities.map((elem) => ({
        name: elem.name,
        id: elem.id,
      }));
    }
  }

  startEditActivityRow(row: TimesheetActivityHours) {
    this.activityUnderEdit = row.activityId;
    this.editActivityForm.setValue({
      companyId: row.companyId,
      projectId: row.projectId,
      activityId: row.activityId,
      hours: row.hours,
      notes: row.notes,
    });
    this.updateProjectsOfCompany();
  }

  confirmEditActivityRow(row: TimesheetActivityHours, index: number) {
    this.activityUnderEdit = null;
    var activityToEdit = this.activities[index];
    activityToEdit.companyId = this.editActivityForm.value['companyId'];
    activityToEdit.projectId = this.editActivityForm.value['projectId'];
    activityToEdit.activityId = this.editActivityForm.value['activityId'];
    activityToEdit.notes = this.editActivityForm.value['notes'];
    activityToEdit.hours = this.editActivityForm.value['hours'];
    activityToEdit.companyName = this.companies.filter(
      (elem) => elem.id === activityToEdit.companyId
    )[0].name;
    activityToEdit.projectName = this.projectsOfCompany.filter(
      (elem) => elem.id === activityToEdit.projectId
    )[0].name;
    activityToEdit.activityName = this.activitiesOfProject.filter(
      (elem) => elem.id === activityToEdit.activityId
    )[0].name;

    this.editActivityForm.setValue({
      companyId: null,
      projectId: null,
      activityId: null,
      hours: 0,
      notes: '',
    });
    this.updateProjectsOfCompany();
  }

  cancelEditActivityRow(row: TimesheetActivityHours) {
    this.activityUnderEdit = null;
    this.editActivityForm.setValue({
      companyId: null,
      projectId: null,
      activityId: null,
      hours: 0,
      notes: '',
    });
    this.activitiesTableDataSource = new MatTableDataSource(this.activities);
    this.updateProjectsOfCompany();
  }

  deleteActivityRow(row: TimesheetActivityHours, index: number) {
    this.activities.splice(index, 1);
    this.activitiesTableDataSource = new MatTableDataSource(this.activities);
  }

  addNewActivity() {
    this.activities.push({
      companyId: '',
      projectId: '',
      activityId: 'tmpActivityId',
      companyName: '',
      projectName: '',
      activityName: '',
      notes: '',
      hours: 0,
    });
    this.activityUnderEdit = 'tmpActivityId';
    this.editActivityForm.setValue({
      companyId: null,
      projectId: null,
      activityId: 'edit me',
      hours: 0,
      notes: '',
    });
    this.activitiesTableDataSource = new MatTableDataSource(this.activities);
  }

  onSubmit() {
    const dayType = this.dayForm.value['dayType'];

    if (
      dayType === TimesheetDayWorkType.Office ||
      dayType === TimesheetDayWorkType.Remote
    ) {
      this.dialogRef.close({
        dayType: dayType,
        permitHours: this.dayForm.value['permitHours'],
        activities: this.activities,
      });
    } else {
      this.dialogRef.close({
        dayType: dayType,
        permitHours: 0,
        activities: [],
      });
    }
  }
}
