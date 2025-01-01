import { CommonModule } from '@angular/common';
import { Component, ViewEncapsulation } from '@angular/core';
import {
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { CreateUserParameters } from '../../../../types/model';

@Component({
  selector: 'new-user-modal',
  templateUrl: './new-user-modal.html',
  styleUrls: ['./new-user-modal.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatButtonModule,
    MatInputModule,
    MatDialogModule,
    MatFormFieldModule,
    ReactiveFormsModule,
  ],
  encapsulation: ViewEncapsulation.None,
})
export class NewUserDialogComponent {
  newUserForm: FormGroup = this.formBuilder.group({
    username: ['', Validators.required],
    password: ['', Validators.required],
    passwordConfirm: ['', Validators.required],
    name: ['', Validators.required],
    surname: ['', Validators.required],
    email: ['', Validators.required],
  });

  constructor(
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<CreateUserParameters>
  ) {}

  onSubmit() {
    this.dialogRef.close({
      username: this.newUserForm.value['username'],
      password: this.newUserForm.value['password'],
      name: this.newUserForm.value['name'],
      surname: this.newUserForm.value['surname'],
      email: this.newUserForm.value['email'],
    });
  }
}
