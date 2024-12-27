import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { ApiService } from '../../../service/api.service';

@Component({
  selector: 'service-page',
  templateUrl: './service.html',
  styleUrls: ['./service.scss'],
  standalone: true,
  imports: [CommonModule, MatProgressSpinnerModule],
  encapsulation: ViewEncapsulation.None,
})
export class ServicePageComponent implements OnInit {
  //1)define variable to hold "isLoading"
  loading: boolean = false;
  //define variable to handle "ok" and "ko" case, in this case null is equal to KO
  apiResult: string | null = null;

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.loadData();
  }

  loadData() {
    this.loading = true;
    this.apiService.getApiStringExample().subscribe({
      next: (response) => {
        this.apiResult = response;
        this.loading = false;
      },
      error: () => {
        this.apiResult = null;
        this.loading = false;
      },
    });
  }
}
