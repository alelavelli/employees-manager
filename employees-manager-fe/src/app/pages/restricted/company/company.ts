import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
@Component({
  selector: 'company-page',
  templateUrl: './company.html',
  styleUrls: ['./company.scss'],
  standalone: true,
  imports: [CommonModule],
})
export class CompanyPageComponent implements OnInit {
  constructor() {}

  ngOnInit(): void {}
}
