import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
@Component({
  selector: 'timesheet-page',
  templateUrl: './timesheet.html',
  styleUrls: ['./timesheet.scss'],
  standalone: true,
  imports: [CommonModule],
})
export class TimesheetPageComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
