import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
@Component({
  selector: 'calendar-page',
  templateUrl: './calendar.html',
  styleUrls: ['./calendar.scss'],
  standalone: true,
  imports: [CommonModule],
})
export class CalendarPageComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
