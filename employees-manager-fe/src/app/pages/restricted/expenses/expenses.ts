import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
@Component({
  selector: 'expenses-page',
  templateUrl: './expenses.html',
  styleUrls: ['./expenses.scss'],
  standalone: true,
  imports: [CommonModule],
})
export class ExpensesPageComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
