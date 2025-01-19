import { CommonModule } from '@angular/common';
import {
  Component,
  Inject,
  Input,
  OnInit,
  ViewEncapsulation,
} from '@angular/core';
import {
  AbstractControl,
  AbstractControlOptions,
  FormBuilder,
  FormControl,
  FormGroup,
  ReactiveFormsModule,
  ValidationErrors,
  ValidatorFn,
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
import { InviteUserInCompany, UserToInvite } from '../../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { CompanyRole } from '../../../../types/enums';
import { AsyncPipe } from '@angular/common';
import { MatAutocompleteModule } from '@angular/material/autocomplete';
import { Observable, startWith, map, of } from 'rxjs';
import { ApiService } from '../../../../service/api.service';

@Component({
  selector: 'invite-user-modal',
  templateUrl: './invite-user-modal.html',
  styleUrls: ['./invite-user-modal.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatIconModule,
    MatButtonModule,
    MatInputModule,
    MatDialogModule,
    MatFormFieldModule,
    MatSelectModule,
    ReactiveFormsModule,
    MatAutocompleteModule,
    AsyncPipe,
  ],
  encapsulation: ViewEncapsulation.None,
})
export class InviteUserInCompanyDialogComponent implements OnInit {
  companyId: string | null;
  CompanyRole = CompanyRole;
  invitationForm: FormGroup = this.formBuilder.group({
    username: ['', Validators.required],
    role: ['', Validators.required],
    jobTitle: ['', Validators.required],
  });
  usersToInvite: UserToInvite[] = [];
  filteredUsers: Observable<UserToInvite[]>;

  constructor(
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<InviteUserInCompany>,
    private apiService: ApiService,
    @Inject(MAT_DIALOG_DATA)
    public data: { companyId: string; role: CompanyRole }
  ) {
    this.companyId = data.companyId;
    this.filteredUsers = of([]);
  }

  ngOnInit(): void {
    this.apiService.getUsersToInvite(this.companyId!).subscribe({
      next: (response) => {
        this.usersToInvite = response;
        const usernameField = this.invitationForm.get('username');
        if (usernameField) {
          usernameField.setValidators([
            Validators.required,
            this.existUsernameValidator(
              this.usersToInvite.map((user) => user.username)
            ),
          ]);
          usernameField.updateValueAndValidity();
        }
      },
    });
    this.filteredUsers = this.invitationForm.valueChanges.pipe(
      startWith(''),
      map((value: { username: string }) => {
        const name =
          typeof value.username === 'string' ? value.username : value.username!;
        return name
          ? this._filterUsername(name as string)
          : this.usersToInvite.slice();
      })
    );
  }

  existUsernameValidator(usernames: string[]): ValidatorFn {
    return (control: AbstractControl): ValidationErrors | null => {
      const is_valid = usernames.includes(control.value);
      return is_valid ? null : { username: { value: control.value } };
    };
  }

  private _filterUsername(name: string): UserToInvite[] {
    const filterValue = name.toLowerCase();

    return this.usersToInvite.filter((option) =>
      option.username.toLowerCase().includes(filterValue)
    );
  }

  onSubmit() {
    this.dialogRef.close({
      userId: this.usersToInvite.filter(
        (elem) => elem.username === this.invitationForm.value['username']
      )[0].userId,
      role: this.invitationForm.value['role'],
      jobTitle: this.invitationForm.value['jobTitle'],
    });
  }
}
