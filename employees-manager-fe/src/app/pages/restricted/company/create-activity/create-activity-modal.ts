import { CommonModule } from '@angular/common';
import { Component, Inject, OnInit, ViewEncapsulation } from '@angular/core';
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
import { InviteUserInCompany } from '../../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { AsyncPipe } from '@angular/common';
import { MatAutocompleteModule } from '@angular/material/autocomplete';

@Component({
  selector: 'create-activity-modal',
  templateUrl: './create-activity-modal.html',
  styleUrls: ['./create-activity-modal.scss'],
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
export class NewActivityDialogComponent implements OnInit {
  companyId: string | null;
  newActivityForm: FormGroup = this.formBuilder.group({
    name: ['', Validators.required],
    description: [''],
  });

  constructor(
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<InviteUserInCompany>,
    @Inject(MAT_DIALOG_DATA)
    public data: { companyId: string }
  ) {
    this.companyId = data.companyId;
  }

  ngOnInit(): void {}

  onSubmit() {
    this.dialogRef.close({
      name: this.newActivityForm.value['name'],
      description: this.newActivityForm.value['description'],
    });
  }
}
