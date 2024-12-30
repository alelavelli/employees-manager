import { CommonModule } from '@angular/common';
import { Component, OnInit, ViewEncapsulation } from '@angular/core';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { ApiService } from '../../../service/api.service';
import { AdminPanelOverview } from '../../../types/model';

@Component({
  selector: 'admin-page',
  templateUrl: './admin.html',
  styleUrls: ['./admin.scss'],
  standalone: true,
  imports: [CommonModule, MatProgressBarModule],
  encapsulation: ViewEncapsulation.None,
})
export class AdminPageComponent implements OnInit {
  //1)define variable to hold "isLoading"
  loading: boolean = false;
  //define variable to handle "ok" and "ko" case, in this case null is equal to KO
  overview: AdminPanelOverview | null = null;

  constructor(private apiService: ApiService) {}

  ngOnInit(): void {
    this.loadOverview();
  }

  loadOverview() {
    this.loading = true;
    this.apiService.getAdminPanelOverview().subscribe({
      next: (response) => {
        this.overview = response;
        this.loading = false;
      },
      error: () => {
        this.overview = null;
        this.loading = false;
      },
    });
  }
}
