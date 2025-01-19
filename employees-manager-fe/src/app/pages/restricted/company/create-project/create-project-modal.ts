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
  selector: 'create-project-modal',
  templateUrl: './create-project-modal.html',
  styleUrls: ['./create-project-modal.scss'],
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
export class NewCompanyProjectDialogComponent implements OnInit {
  companyId: string | null;
  newProjectForm: FormGroup = this.formBuilder.group({
    name: ['', Validators.required],
    code: ['', Validators.required],
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
      name: this.newProjectForm.value['name'],
      code: this.newProjectForm.value['code'],
    });
  }
}
