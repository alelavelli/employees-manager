import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  Inject,
  OnInit,
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
import { MatIconModule } from '@angular/material/icon';
import { MatAutocompleteModule } from '@angular/material/autocomplete';
import { MatChipsModule } from '@angular/material/chips';
import { ApiService } from '../../../../service/api.service';
import { CreateCorporateGroupParameters } from '../../../../types/model';

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

  constructor(
    private apiService: ApiService,
    private formBuilder: FormBuilder,
    public dialogRef: MatDialogRef<CreateCorporateGroupParameters>,
    @Inject(MAT_DIALOG_DATA)
    public data: { userId: string }
  ) {
    this.userId = data.userId;
  }

  ngOnInit(): void {}

  onSubmit() {
    this.dialogRef.close({
      name: this.newCorporateGroupForm.value['name'],
    });
  }
}
