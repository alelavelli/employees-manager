import { CommonModule } from '@angular/common';
import { Component, ViewEncapsulation } from '@angular/core';
import {
  AbstractControlOptions,
  FormBuilder,
  FormControl,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { CreateCompanyParameters } from '../../../../types/model';
import { MatIconModule } from '@angular/material/icon';

@Component({
  selector: 'new-company-modal',
  templateUrl: './new-company-modal.html',
  styleUrls: ['./new-company-modal.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatIconModule,
    MatButtonModule,
    MatInputModule,
    MatDialogModule,
    MatFormFieldModule,
    ReactiveFormsModule,
  ],
  encapsulation: ViewEncapsulation.None,
})
export class NewCompanyDialogComponent {
  newCompanyForm: FormGroup = this.formBuilder.group({
    name: ['', Validators.required],
    jobTitle: ['', Validators.required],
  });

  constructor(
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<CreateCompanyParameters>
  ) {}

  onSubmit() {
    this.dialogRef.close({
      name: this.newCompanyForm.value['name'],
      jobTitle: this.newCompanyForm.value['jobTitle'],
    });
  }
}
