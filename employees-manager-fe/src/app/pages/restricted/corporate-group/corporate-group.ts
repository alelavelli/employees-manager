import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { CorporateGroupInfo } from '../../../types/model';
import { MatTableDataSource, MatTableModule } from '@angular/material/table';
import { FormBuilder, FormGroup, ReactiveFormsModule } from '@angular/forms';
import { ApiService } from '../../../service/api.service';
@Component({
  selector: 'corporate-group-page',
  templateUrl: './corporate-group.html',
  styleUrls: ['./corporate-group.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatProgressBarModule,
    MatFormFieldModule,
    MatIconModule,
    MatButtonModule,
    MatTableModule,
    ReactiveFormsModule,
  ],
})
export class CorporateGroupPageComponent implements OnInit {
  loading: boolean = false;

  constructor(
    private apiService: ApiService,
    private formBuilder: FormBuilder
  ) {}

  ngOnInit(): void {
    this.loadData();
  }

  loadData() {}

  openNewGroupDialog() {}
}
