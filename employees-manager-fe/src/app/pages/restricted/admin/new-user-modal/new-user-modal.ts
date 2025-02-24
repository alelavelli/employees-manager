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
import { CreateUserParameters } from '../../../../types/model';
import { MatIconModule } from '@angular/material/icon';

@Component({
  selector: 'new-user-modal',
  templateUrl: './new-user-modal.html',
  styleUrls: ['./new-user-modal.scss'],
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
export class NewUserDialogComponent {
  newUserForm: FormGroup = this.formBuilder.group(
    {
      username: ['', Validators.required],
      password: new FormControl('', [
        Validators.required,
        Validators.minLength(8),
        Validators.pattern(/[a-z]/), // At least 1 lowercase
        Validators.pattern(/[A-Z]/), // At least 1 uppercase
        Validators.pattern(/\d/), // At least 1 digit
        Validators.pattern(/[^a-zA-Z0-9]/), // At least 1 symbol
      ]),
      confirmPassword: new FormControl('', [Validators.required]),
      name: ['', Validators.required],
      surname: ['', Validators.required],
      email: ['', [Validators.required, Validators.pattern(/\S+@\S+\.\S+/)]],
    },
    <AbstractControlOptions>{ validators: [this.passwordMatchValidator] }
  );

  passwordConstraints = [
    { respected: false, message: 'at least 8 characters long' },
    { respected: false, message: 'at least one lowercase letter' },
    { respected: false, message: 'at least one uppercase letter' },
    { respected: false, message: 'at least one digit' },
    { respected: false, message: 'at least one symbol' },
    { respected: false, message: 'passwords must match' },
  ];

  constructor(
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<CreateUserParameters>
  ) {
    this.newUserForm.valueChanges.subscribe((form) => {
      const password: string = form.password;
      this.passwordConstraints[0].respected = password.length >= 8;
      this.passwordConstraints[1].respected = password.match(/[a-z]/) !== null;
      this.passwordConstraints[2].respected = password.match(/[A-Z]/) !== null;
      this.passwordConstraints[3].respected = password.match(/\d/) !== null;
      this.passwordConstraints[4].respected =
        password.match(/[^a-zA-Z0-9]/) !== null;
      this.passwordConstraints[5].respected = password === form.confirmPassword;
    });
  }

  passwordMatchValidator(formGroup: FormGroup) {
    const password: string = formGroup.get('password')!.value;
    const confirmPassword: string = formGroup.get('confirmPassword')!.value;

    if (password !== confirmPassword) {
      formGroup.get('confirmPassword')!.setErrors({ passwordMismatch: true });
    } else {
      formGroup.get('confirmPassword')!.setErrors(null);
    }
  }

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
