import { CommonModule } from '@angular/common';
import { Component, ViewEncapsulation } from '@angular/core';
import {
  FormBuilder,
  FormControl,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { Router, RouterModule } from '@angular/router';
import { ApiService } from '../../../service/api.service';
import { UserService } from '../../../service/user.service';
import { MatButtonModule } from '@angular/material/button';
import { LoginResponse } from '../../../types/model';

@Component({
  selector: 'login-page',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.scss'],
  encapsulation: ViewEncapsulation.None,
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatIconModule,
    RouterModule,
    MatButtonModule,
  ],
})
export class LoginPageComponent {
  loading: boolean = false;
  loginForm: FormGroup = this.fb.group({
    username: new FormControl('', [Validators.required]),
    password: new FormControl('', [Validators.required]),
  });
  hidePassword: boolean = true;

  constructor(
    private apiService: ApiService,
    private userService: UserService,
    private router: Router,
    private fb: FormBuilder
  ) {}

  onSubmit() {
    this.loading = true;
    const username = this.loginForm.get('username')!.value;
    const password = this.loginForm.get('password')!.value;

    this.apiService.login(username, password).subscribe({
      next: (loginResponse: LoginResponse) => {
        this.userService.setJwtToken(loginResponse.token);
        this.userService.fetchUserData().subscribe({
          next: () => {
            this.router.navigateByUrl('/home');
            this.loading = false;
          },
          error: () => {
            this.loading = false;
          },
        });
      },
      error: () => {
        this.loading = false;
      },
    });
  }
}
