import { CommonModule } from '@angular/common';
import {
  Component,
  Inject,
  Input,
  OnInit,
  ViewEncapsulation,
  ChangeDetectionStrategy,
  computed,
  inject,
  model,
  signal,
} from '@angular/core';
import {
  AbstractControl,
  AbstractControlOptions,
  FormBuilder,
  FormControl,
  FormGroup,
  FormsModule,
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
import {
  CompanyProjectInfo,
  InviteUserInCompany,
  UserToInvite,
} from '../../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { CompanyRole } from '../../../../types/enums';
import { AsyncPipe } from '@angular/common';
import {
  MatAutocompleteModule,
  MatAutocompleteSelectedEvent,
} from '@angular/material/autocomplete';
import { Observable, startWith, map, of } from 'rxjs';
import { ApiService } from '../../../../service/api.service';
import { MatChipInputEvent, MatChipsModule } from '@angular/material/chips';
import { COMMA, ENTER } from '@angular/cdk/keycodes';
import { LiveAnnouncer } from '@angular/cdk/a11y';

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
    MatChipsModule,
    AsyncPipe,
    FormsModule,
  ],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
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

  separatorKeysCodes: number[] = [ENTER, COMMA];
  currentProject = model('');
  projects = signal([] as string[]);
  allProjects: CompanyProjectInfo[] = [];
  filteredProjects = computed(() => {
    const currentProject = this.currentProject().toLocaleLowerCase();
    const selectedProjects = new Set(this.projects());
    return currentProject
      ? this.allProjects
          .filter((project) =>
            project.name.toLocaleLowerCase().includes(currentProject)
          )
          .filter((project) => !selectedProjects.has(project.name))
      : this.allProjects
          .slice()
          .filter((project) => !selectedProjects.has(project.name));
  });

  announcer = inject(LiveAnnouncer);

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

    this.apiService.getCompanyProjects(this.companyId!).subscribe({
      next: (response) => {
        this.allProjects = response.filter((project) => project.active);
        this.filteredProjects = computed(() => {
          const currentProject = this.currentProject().toLocaleLowerCase();
          const selectedProjects = new Set(this.projects());
          return currentProject
            ? this.allProjects
                .filter((project) =>
                  project.name.toLocaleLowerCase().includes(currentProject)
                )
                .filter((project) => !selectedProjects.has(project.name))
            : this.allProjects
                .slice()
                .filter((project) => !selectedProjects.has(project.name));
        });
      },
    });
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

  add(event: MatChipInputEvent): void {
    const value = (event.value || '').trim();
    if (value) {
      this.projects.update((projects) => [...projects, value]);
    }

    this.currentProject.set('');
  }

  remove(project: string): void {
    this.projects.update((projects) => {
      const index = projects.indexOf(project);
      if (index < 0) {
        return projects;
      }

      projects.splice(index, 1);
      this.announcer.announce(`${project} removed`);
      return [...projects];
    });
  }

  selected(event: MatAutocompleteSelectedEvent): void {
    this.projects.update((projects) => [...projects, event.option.viewValue]);
    this.currentProject.set('');
    event.option.deselect();
  }

  onSubmit() {
    this.dialogRef.close({
      userId: this.usersToInvite.filter(
        (elem) => elem.username === this.invitationForm.value['username']
      )[0].userId,
      role: this.invitationForm.value['role'],
      jobTitle: this.invitationForm.value['jobTitle'],
      projectIds: this.allProjects
        .filter((project) => this.projects().includes(project.name))
        .map((project) => project.id),
    });
  }
}
