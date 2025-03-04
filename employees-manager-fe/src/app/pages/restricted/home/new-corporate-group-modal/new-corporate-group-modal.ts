import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  Inject,
  inject,
  model,
  OnInit,
  signal,
  ViewEncapsulation,
} from '@angular/core';
import {
  FormBuilder,
  FormGroup,
  FormsModule,
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
import {
  CorporateGroupCompanyInfo,
  CreateCompanyParameters,
} from '../../../../types/model';
import { MatIconModule } from '@angular/material/icon';
import {
  MatAutocompleteModule,
  MatAutocompleteSelectedEvent,
} from '@angular/material/autocomplete';
import { MatChipInputEvent, MatChipsModule } from '@angular/material/chips';
import { COMMA, ENTER } from '@angular/cdk/keycodes';
import { LiveAnnouncer } from '@angular/cdk/a11y';
import { of } from 'rxjs';
import { ApiService } from '../../../../service/api.service';

@Component({
  selector: 'new-corporate-group-modal',
  templateUrl: './new-corporate-group-modal.html',
  styleUrls: ['./new-corporate-group-modal.scss'],
  standalone: true,
  imports: [
    CommonModule,
    MatIconModule,
    MatButtonModule,
    MatInputModule,
    MatDialogModule,
    MatFormFieldModule,
    ReactiveFormsModule,
    MatAutocompleteModule,
    MatChipsModule,
    FormsModule,
  ],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NewCorporateGroupDialogComponent implements OnInit {
  userId: null | string;
  newCorporateGroupForm: FormGroup = this.formBuilder.group({
    name: ['', Validators.required],
  });
  separatorKeysCodes: number[] = [ENTER, COMMA];

  currentCompany = model('');
  companies = signal([] as string[]);
  allCompanies: CorporateGroupCompanyInfo[] = [];
  filteredCompanies = computed(() => {
    const currentCompany = this.currentCompany().toLocaleLowerCase();
    const selectedCompanies = new Set(this.companies());
    return currentCompany
      ? this.allCompanies
          .filter((company) =>
            company.name.toLocaleLowerCase().includes(currentCompany)
          )
          .filter((company) => !selectedCompanies.has(company.name))
      : this.allCompanies
          .slice()
          .filter((company) => !selectedCompanies.has(company.name));
  });
  announcer = inject(LiveAnnouncer);

  constructor(
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<CreateCompanyParameters>,
    @Inject(MAT_DIALOG_DATA)
    public data: { userId: string }
  ) {
    this.userId = data.userId;
  }

  ngOnInit(): void {
    this.apiService.getEligibleCompaniesForCorporateGroup().subscribe({
      next: (response) => {
        this.allCompanies = response;
        this.filteredCompanies = computed(() => {
          const currentCompany = this.currentCompany().toLocaleLowerCase();
          const selectedCompanies = new Set(this.companies());
          return currentCompany
            ? this.allCompanies
                .filter((company) =>
                  company.name.toLocaleLowerCase().includes(currentCompany)
                )
                .filter((company) => !selectedCompanies.has(company.name))
            : this.allCompanies
                .slice()
                .filter((company) => !selectedCompanies.has(company.name));
        });
      },
    });
  }

  add(event: MatChipInputEvent): void {
    const value = (event.value || '').trim();
    if (value) {
      this.companies.update((companies) => [...companies, value]);
    }

    this.currentCompany.set('');
  }

  remove(company: string): void {
    this.companies.update((companies) => {
      const index = companies.indexOf(company);
      if (index < 0) {
        return companies;
      }

      companies.splice(index, 1);
      this.announcer.announce(`${company} removed`);
      return [...companies];
    });
  }

  selected(event: MatAutocompleteSelectedEvent): void {
    this.companies.update((companies) => [
      ...companies,
      event.option.viewValue,
    ]);
    this.currentCompany.set('');
    event.option.deselect();
  }

  selectedCompanies(): number {
    return this.allCompanies
      .filter((company) => this.companies().includes(company.name))
      .map((company) => company.id).length;
  }

  onSubmit() {
    this.dialogRef.close({
      name: this.newCorporateGroupForm.value['name'],
      companyIds: this.allCompanies
        .filter((company) => this.companies().includes(company.name))
        .map((company) => company.id),
    });
  }
}
